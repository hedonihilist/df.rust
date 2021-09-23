mod cli;
mod mountinfo;
mod table;

use crate::cli::parse_args;
use crate::table::FieldAlign::{Left, Right};
use cli::Options;
use mountinfo::MountInfo;
use nix;
use nix::sys::statvfs::Statvfs;
use std::collections::{HashMap, HashSet};
use std::fs::read_to_string;
use std::process::id;
use table::{FieldAlign, Table};

#[derive(Debug, Default)]
struct FsUsage {
    source: String,
    fstype: String,
    file: String,
    target: String,
    itotal: u64,
    iused: u64,
    iavail: u64,
    ipcent: u32,
    size: u64,
    used: u64,
    avail: u64,
    pcent: u32,
}

impl FsUsage {
    fn new() -> FsUsage {
        FsUsage {
            source: "-".to_owned(),
            fstype: "-".to_owned(),
            file: "-".to_owned(),
            target: "-".to_owned(),
            ..Default::default()
        }
    }
}

/**
 * a/b = ?%
 */
fn percent_round_up(a: u64, b: u64) -> u32 {
    if 100u64 * a % b == 0 {
        (100u64 * a / b) as u32
    } else {
        1 + (100u64 * a / b) as u32
    }
}

fn fieldname_to_label(s: &str) -> &str {
    match s {
        "source" => "Filesystem",
        "fstype" => "Type",
        "file" => "File",
        "target" => "Mounted on",
        "itotal" => "Inodes",
        "iused" => "IUsed",
        "iavail" => "IFree",
        "ipcent" => "IUse%",
        "size" => "1K-blocks",
        "used" => "Used",
        "avail" => "Avail",
        "pcent" => "Use%",
        _ => "",
    }
}

fn get_dev(mount: MountInfo, options: &Options) -> Option<FsUsage> {
    if mount.is_remote() && options.show_local_fs {
        return None;
    }
    if mount.is_dummy() && !options.show_all_fs && options.listed_fs.is_empty() {
        return None;
    }
    // fs_type not listed
    if !options.listed_fs.is_empty() && !options.listed_fs.contains(&mount.fs_type)
        || options.excluded_fs.contains(&mount.fs_type)
    {
        return None;
    }

    // TODO
    // grand total
    let mut fs_usage = FsUsage::new();

    // stat the fs
    if let Ok(stat) = nix::sys::statvfs::statvfs::<str>(mount.mount_point.as_ref()) {
        // 信息读取成功
        // 填充各种信息
        // block
        fs_usage.size = stat.blocks();
        fs_usage.used = stat.blocks() - stat.blocks_free();
        fs_usage.avail = stat.blocks_available();
        // 显示时按照1K大小的块显示
        fs_usage.size = fs_usage.size * stat.fragment_size() / 1024;
        fs_usage.used = fs_usage.used * stat.fragment_size() / 1024;
        fs_usage.avail = fs_usage.avail * stat.fragment_size() / 1024;
        // round up
        fs_usage.pcent = match fs_usage.size != 0 {
            true => percent_round_up(fs_usage.used, fs_usage.used + fs_usage.avail), // 不是除fs_usage.size，见coreutils中的df
            false => 0,
        };
        // inode
        fs_usage.itotal = stat.files();
        fs_usage.iused = stat.files() - stat.files_available();
        fs_usage.iavail = stat.files_available();
        fs_usage.ipcent = match fs_usage.itotal != 0 {
            true => percent_round_up(fs_usage.iused, fs_usage.iused + fs_usage.iavail),
            false => 0,
        };

        fs_usage.fstype = mount.fs_type;
        fs_usage.source = mount.mount_source;
        fs_usage.target = mount.mount_point;
    } else {
        // TODO 判断是不是权限原因
        return None;
    }

    if fs_usage.size == 0 && !options.show_all_fs && options.listed_fs.is_empty() {
        return None;
    }

    Some(fs_usage)
}


fn options_to_fields(options: &Options) -> Vec<String> {
    if options.output_all_fields {
        return vec![
            "Filesystem",
            "Type",
            "Inodes",
            "IUsed",
            "IFree",
            "IUse%",
            "1K-blocks",
            "Used",
            "Avail",
            "Use%",
            "File",
            "Mounted on",
        ].iter().map(|x| x.to_string()).collect();
    }
    if !options.field_list.is_empty() {
        let mut fields = vec![];
        for f in options.field_list.iter() {
            let name = fieldname_to_label(f);
            if name.is_empty() {
                panic!("no such field: {}", f);
            }
            fields.push(name.to_owned());
        }
        return fields
    }
    // 没有写明fields, 默认现实block，写明了inode则显示inodes
    if options.inodes {
        return vec![
            "Filesystem",
            "Inodes",
            "IUsed",
            "IFree",
            "IUse%",
            "Mounted on",
        ].iter().map(|x| x.to_string()).collect();
    }
    vec![
        "Filesystem",
        "1K-blocks",
        "Used",
        "Avail",
        "Use%",
        "File",
        "Mounted on",
    ].iter().map(|x| x.to_string()).collect()
}

fn show_table(options: &Options, table: &Table) {
    let fields = options_to_fields(options);
    println!("{}", table.to_string_partial(&fields));
}

fn get_all_entries(options: &Options) -> Table {
    let mountlist = filter_mountinfo_list(mountinfo::get_mountinfo_list(), options);

    let fields = vec![
        "Filesystem",
        "Type",
        "Inodes",
        "IUsed",
        "IFree",
        "IUse%",
        "1K-blocks",
        "Used",
        "Avail",
        "Use%",
        "File",
        "Mounted on",
    ];
    let align_list = vec![
        Left, Left, Right, Right, Right, Right, Right, Right, Right, Right, Left, Left,
    ];
    let mut table = Table::new(&fields);
    for (i, align) in align_list.into_iter().enumerate() {
        table.set_field_align(fields[i], align);
    }

    // get fs usage
    for mount in mountlist.into_iter() {
        if let Some(fsu) = get_dev(mount, options) {
            // populate a table
            let mut row: Vec<String> = vec![];

            // source
            row.push(fsu.source);

            // fs_type
            row.push(fsu.fstype);

            // Inodes
            row.push(fsu.itotal.to_string());

            // Inode used
            row.push(fsu.iused.to_string());

            // Inode free
            row.push(fsu.iavail.to_string());

            // Inode percent
            if fsu.itotal == 0 {
                row.push("-".to_string());
            } else {
                row.push(fsu.ipcent.to_string() + "%");
            }

            // blocks
            row.push(fsu.size.to_string());

            // block used
            row.push(fsu.used.to_string());

            // block available
            row.push(fsu.avail.to_string());

            // block used percent
            row.push(fsu.pcent.to_string() + "%");

            // TODO File
            row.push("-".to_string());

            // mount point
            row.push(fsu.target.to_string());

            table.add_row(&row);
        }
    }

    table
}

/**
 * df.c中的filter_mountinfo_list的作用是去重，而不是根据输入的参数过滤掉mountinfo
 * 真正的过滤在get_dev中
 */
fn filter_mountinfo_list(list: Vec<MountInfo>, options: &Options) -> Vec<MountInfo> {
    let mut filtered: Vec<MountInfo> = vec![];
    let mut seen: HashMap<u64, usize> = HashMap::new();
    for me in list.into_iter() {
        let mut discard_me: Option<usize> = None; //
                                                  // skip
        if (me.is_remote() && options.show_local_fs)
            || (me.is_dummy() && !options.show_all_fs && !options.listed_fs.contains(&me.fs_type))
            || (!options.listed_fs.is_empty() && !options.listed_fs.contains(&me.fs_type))
            || options.excluded_fs.contains(&me.fs_type)
        {
            // pass
        } else {
            /*
            在Linux中有一个bind mount的概念，能把一个目录挂载到另一个目录下，例如mount -o bind /boot/efi /tmp/bindmount
            在df的输出中我们希望去除重复的设备
             */
            if let Some(&idx) = seen.get(&me.dev()) {
                let seen_dev: &MountInfo = &filtered[idx];

                // target指当前me
                // source指
                let target_nearer_root = seen_dev.mount_point.len() > me.mount_point.len();
                let source_below_root = !seen_dev.root.is_empty()
                    && !me.root.is_empty()
                    && (seen_dev.root.len() < me.root.len());
                if (!options.print_grand_total
                    && me.is_remote()
                    && seen_dev.is_remote()
                    && seen_dev.mount_source.eq(&me.mount_source))
                {
                    // don't discard
                } else if (me.mount_source.contains('/') && !seen_dev.mount_source.contains('/'))
                    || (target_nearer_root && !source_below_root)
                    || (!seen_dev.mount_source.eq(&me.mount_source)
                        && seen_dev.mount_point.eq(&me.mount_point))
                {
                    // discard this one
                    continue;
                } else {
                    discard_me = Some(idx);
                }
            }
        }
        if let Some(discard_idx) = discard_me {
            std::mem::replace(&mut filtered[discard_idx], me);
        } else {
            let dev = me.dev();
            filtered.push(me);
            seen.insert(dev, filtered.len() - 1);
        }
    }
    filtered
}

fn main() {
    //println!("{:?}", cli::parse_args());
    let options = parse_args();
    let table = get_all_entries(&options);
    if table.is_empty() {
        println!("no file systems processed");
        return;
    }
    show_table(&options, &table);
}

mod mountinfo;
mod table;

use nix;
use mountinfo::MountInfo;
use std::collections::{HashMap, HashSet};
use std::process::id;
use table::Table;
use std::fs::read_to_string;
use nix::sys::statvfs::Statvfs;

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


fn get_dev(mount: MountInfo, options: &Options) -> Option<FsUsage> {
    if mount.is_remote() && options.show_local_fs {
        return None;
    }
    if mount.is_dummy() && !options.show_all_fs && options.listed_fs.is_empty() {
        return None;
    }
    // fs_type not listed
    if !options.listed_fs.is_empty() && !options.listed_fs.contains(&mount.fs_type) || options.excluded_fs.contains(&mount.fs_type) {
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
        fs_usage.avail = stat.blocks_free();
        fs_usage.pcent = match fs_usage.size != 0 {
            true => (100u64 * fs_usage.used / fs_usage.size) as u32,
            false => 0,
        };
        // inode
        fs_usage.itotal = stat.files();
        fs_usage.iused = stat.files() - stat.files_free();
        fs_usage.avail = stat.files_free();
        fs_usage.ipcent = match fs_usage.itotal != 0 {
            true => (100u64 * fs_usage.iused / fs_usage.itotal) as u32,
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

fn get_all_entries(options: &Options) -> Table {
    let mountlist = filter_mountinfo_list(mountinfo::get_mountinfo_list(), options);

    // decide the fields

    // get fs usage
    for mount in mountlist.into_iter() {
        if let Some(fsu) = get_dev(mount, options) {
            println!("{:?}", &fsu);
            // convert to
        } else {
            //println!("ignored");
        }
    }

    // store in table
    Table::new(&vec![""])
}

#[derive(Default)]
struct Options {
    show_local_fs: bool,
    show_all_fs: bool,
    listed_fs: HashSet<String>,
    excluded_fs: HashSet<String>,
    human_readable: bool, // true => powers of 1024, false => powers of 1000
    print_grand_total: bool,
    field_list: Vec<String>,
}

impl Options {
    fn new() -> Options {
        Options {
            show_local_fs: true,
            show_all_fs: false,
            human_readable: true,
            ..Default::default()
        }
    }
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
    get_all_entries(&Options::default());
}

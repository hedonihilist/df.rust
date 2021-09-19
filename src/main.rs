mod mountinfo;
mod table;

use mountinfo::MountInfo;
use std::collections::{HashMap, HashSet};
use std::process::id;
use table::Table;

struct FsUsage {
    source: String,
    fstype: String,
    itotal: u64,
    iused: u64,
    iavail: u64,
    ipcent: u32,
    size: u64,
    used: u64,
    avail: u64,
    pcent: u32,
    file: String,
    target: String,
}

impl Into<Table> for FsUsage {

}

impl TryFrom<MountInfo> for FsUsage {

}

fn get_dev(mount: MountInfo, options: &Options) -> Option<FsUsage> {

}

fn get_all_entries(options: &Options) -> Table {
    let mountlist = filter_mountinfo_list(mountinfo::get_mountinfo_list(), options);

    // get fs usage
    

    // store in table
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
    let mount = mountinfo::get_mountinfo_list();
    println!("size: {}", mount.len());
    let filtered = filter_mountinfo_list(mount, &Options::new());
    println!("size: {}", filtered.len());
    for mountinfo in filtered.into_iter() {
        println!(
            "{}:{} => {}",
            mountinfo.major_dev, mountinfo.minor_dev, mountinfo.mount_point
        );
    }
}

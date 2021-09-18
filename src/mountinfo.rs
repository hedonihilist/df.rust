use std::fs;

#[derive(Debug, Default)]
pub struct MountInfo {
    pub mount_id: u32,
    pub parent_mount_id: u32,
    pub major_dev: u32,
    pub minor_dev: u32,
    pub root: String,
    pub mount_point: String,
    pub mount_options: Vec<String>,
    pub optional_fields: Option<Vec<String>>,
    pub fs_type: String,
    pub mount_source: String,
    pub super_options: Vec<String>,
}

impl MountInfo {
    pub fn dev(&self) -> u64 {
        // refer to linux/kdev_t.h
        ((self.major_dev as u64) << 8) | self.minor_dev as u64
    }
    /**
     * refer to mountlist.c:read_file_system_list for detailed explaination
     */
    pub fn is_remote(&self) -> bool {
        self.mount_source.contains(':')
            || (self.mount_source.starts_with("//")
                && (&["smbfs", "smb3", "cifs"])
                    .into_iter()
                    .any(|x| x.eq(&self.fs_type)))
            || (&["afs", "auristorfs"])
                .into_iter()
                .any(|x| x.eq(&self.fs_type))
            || self.mount_source.eq("-hosts")
    }

    pub fn is_dummy(&self) -> bool {
        let dummy_fs_type = [
            "autofs",
            "proc",
            "subfs",
            "debugfs",
            "devpts",
            "fusectl",
            "mqueue",
            "rpc_pipefs",
            "sysfs",
            "devfs",
            "kernfs",
            "ignore",
            "none",
        ];
        (&dummy_fs_type).into_iter().any(|x| x.eq(&self.fs_type))
    }
}

/**
 * parse the option string. options are seperated by comma
 */
fn parse_options(options: &str) -> Vec<String> {
    let v: Vec<String> = options.split(",").map(|s| s.to_string()).collect();
    v
}

pub fn get_mountinfo_list() -> Vec<MountInfo> {
    let mut list: Vec<MountInfo> = vec![];
    let content = fs::read_to_string("/proc/self/mountinfo")
        .expect("Error reading mountinfo at /proc/self/mountinfo");
    for line in content.split("\n") {
        if !line.trim().is_empty() {
            let mnt = parse_mountinfo(line);
            list.push(mnt);
        }
    }
    list
}

/**
 * parse a line of mountinfo
 */
pub fn parse_mountinfo(line: &str) -> MountInfo {
    let v: Vec<&str> = line.split(" ").collect();
    let mut mnt = MountInfo::default();
    mnt.mount_id = v[0].parse().unwrap();
    mnt.parent_mount_id = v[1].parse().unwrap();

    // dev num
    let dev: Vec<&str> = v[2].split(":").collect();

    mnt.major_dev = dev[0].parse::<u32>().unwrap();
    mnt.minor_dev = dev[1].parse::<u32>().unwrap();
    mnt.root = v[3].to_string();
    mnt.mount_point = v[4].to_string();
    mnt.mount_options = parse_options(v[5]);
    let mut next_idx: usize = 6;
    let mut fields: Vec<String> = vec![];
    while next_idx < v.len() && v[next_idx].ne("-") {
        fields.push(v[next_idx].to_string());
        next_idx += 1;
    }

    if !fields.is_empty() {
        mnt.optional_fields = Some(fields);
    }

    // skip hyphen
    next_idx += 1;
    if next_idx + 2 >= v.len() {
        panic!("incomplete mountinfo line");
    }

    mnt.fs_type = v[next_idx].to_string();
    next_idx += 1;
    mnt.mount_source = v[next_idx].to_string();
    next_idx += 1;
    mnt.super_options = parse_options(v[next_idx]);
    mnt
}

#[cfg(test)]
mod tests {
    use crate::mountinfo::get_mountinfo_list;
    use crate::mountinfo::parse_mountinfo;

    #[test]
    fn test_parse_mountinfo() {
        let line = "26 29 0:5 / /dev rw,nosuid,relatime shared:2 - devtmpfs dev rw,size=7631200k,nr_inodes=1907800,mode=755,inode64";
        let mntinfo = parse_mountinfo(line);
        assert_eq!(mntinfo.mount_id, 26);
        assert_eq!(mntinfo.parent_mount_id, 29);
        assert_eq!(mntinfo.major_dev, 0);
        assert_eq!(mntinfo.minor_dev, 5);
        assert_eq!(mntinfo.root, "/");
        assert_eq!(mntinfo.mount_point, "/dev");
        assert_eq!(mntinfo.mount_options, ["rw", "nosuid", "relatime"]);
        assert_eq!(mntinfo.optional_fields, Some(vec!["shared:2".to_string()]));
        assert_eq!(mntinfo.fs_type, "devtmpfs");
        assert_eq!(mntinfo.mount_source, "dev");
        assert_eq!(
            mntinfo.super_options,
            [
                "rw",
                "size=7631200k",
                "nr_inodes=1907800",
                "mode=755",
                "inode64"
            ]
        );
        println!("{:?}", mntinfo);
    }

    #[test]
    fn test_no_optional_fields() {
        let line = "26 29 0:5 / /dev rw,nosuid,relatime - devtmpfs dev rw,size=7631200k,nr_inodes=1907800,mode=755,inode64";
        let mntinfo = parse_mountinfo(line);
        assert_eq!(mntinfo.mount_id, 26);
        assert_eq!(mntinfo.parent_mount_id, 29);
        assert_eq!(mntinfo.major_dev, 0);
        assert_eq!(mntinfo.minor_dev, 5);
        assert_eq!(mntinfo.root, "/");
        assert_eq!(mntinfo.mount_point, "/dev");
        assert_eq!(mntinfo.mount_options, ["rw", "nosuid", "relatime"]);
        assert!(mntinfo.optional_fields.is_none());
        assert_eq!(mntinfo.fs_type, "devtmpfs");
        assert_eq!(mntinfo.mount_source, "dev");
        assert_eq!(
            mntinfo.super_options,
            [
                "rw",
                "size=7631200k",
                "nr_inodes=1907800",
                "mode=755",
                "inode64"
            ]
        );
        println!("{:?}", mntinfo);
    }

    #[test]
    fn test_mountinfo_list() {
        let list = get_mountinfo_list();
        println!("{:#?}", list);
        assert!(!list.is_empty());
    }
}

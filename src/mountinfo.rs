#[derive(Debug, Default)]
pub struct MountInfo {
    mount_id: u32,
    parent_mount_id: u32,
    dev: String,
    root: String,
    mount_point: String,
    mount_options: String,
    optional_fields: Option<Vec<String>>,
    fs_type: String,
    mount_source: String,
    super_options: String,
}

fn parse_mountinfo(line: &str) -> MountInfo {
    let v: Vec<&str> = line.split(" ").collect();
    let mut mnt = MountInfo::default();
    mnt.mount_id = v[0].parse().unwrap();
    mnt.parent_mount_id = v[1].parse().unwrap();
    mnt.dev = v[2].to_string();
    mnt.root = v[3].to_string();
    mnt.mount_point = v[4].to_string();
    mnt.mount_options = v[5].to_string();
    let mut next_idx: usize = 6;
    if v[next_idx].ne("-") {
        let mut fields: Vec<String> = vec![];
        while next_idx < v.len() {
            if v[next_idx].eq("-") {
                next_idx += 1; // skip hypen
                break;
            } else {
                fields.push(v[next_idx].to_string());
            }
            next_idx += 1;
        }
        mnt.optional_fields = match fields.is_empty() {
            true => None,
            false => Some(fields),
        };
    }

    mnt.fs_type = v[next_idx].to_string();
    next_idx += 1;
    mnt.mount_source = v[next_idx].to_string();
    next_idx += 1;
    mnt.super_options = v[next_idx].to_string();
    mnt
}

#[cfg(test)]
mod tests {
    use crate::mountinfo::parse_mountinfo;

    #[test]
    fn test_parse_mountinfo() {
        let line = "26 29 0:5 / /dev rw,nosuid,relatime shared:2 - devtmpfs dev rw,size=7631200k,nr_inodes=1907800,mode=755,inode64";
        let mntinfo = parse_mountinfo(line);
        assert_eq!(mntinfo.mount_id, 26);
        assert_eq!(mntinfo.parent_mount_id, 29);
        assert_eq!(mntinfo.dev, "0:5");
        assert_eq!(mntinfo.root, "/");
        assert_eq!(mntinfo.mount_point, "/dev");
        assert_eq!(mntinfo.mount_options, "rw,nosuid,relatime");
        assert_eq!(mntinfo.optional_fields, Some(vec!["shared:2".to_string()]));
        assert_eq!(mntinfo.fs_type, "devtmpfs");
        assert_eq!(mntinfo.mount_source, "dev");
        assert_eq!(
            mntinfo.super_options,
            "rw,size=7631200k,nr_inodes=1907800,mode=755,inode64"
        );
        println!("{:?}", mntinfo)
    }
}

# Reference

## 实现参考

### `df`输出的数据

`df`的输出数据有：

| field  | 含义 |
| ------ | ---- |
| source | 设备 |
|fstype  | 文件系统类型 |
|itotal  | 文件系统中总的inode数 |
|iused   | 已使用的inode数 |
|iavail  | 可使用的inode数 |
|ipcent  | 已使用的inode百分比 |
|size| 文件系统的总block数 |
|used| 已使用的block数 |
|avail| 可用的block数 |
|pcent   | 可用的block百分比 |
|file    | 作为参数输入的文件名 |
|target  | 挂载点 |


各个字段的详细含义见`info df`。





### 如何获取挂载列表

读取`/proc/mounts`文件即可获取挂载列表，文件的格式见`man 5 proc`中的`/proc/self/mountinfo`，`/proc/mounts`是一个指向`/proc/self/mountinfo`的链接。



### 如何获取文件系统的使用情况

从底层看来，使用的是`statfs`这个系统调用，这个系统调用可以获取上面说的所有信息，实现上考虑使用[这个库](https://docs.rs/nix/0.22.1/nix/all.html)。



有关`statfs`以及`statvfs`的区别，查看[这个链接](https://stackoverflow.com/questions/1653163/difference-between-statvfs-and-statfs-system-calls)


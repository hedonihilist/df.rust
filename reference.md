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
|pcent   | 已使用的block百分比 |
|file    | 作为参数输入的文件名 |
|target  | 挂载点 |


各个字段的详细含义见`info df`。





### 如何获取挂载列表

读取`/proc/mounts`文件即可获取挂载列表，文件的格式见`man 5 proc`中的`/proc/self/mountinfo`，`/proc/mounts`是一个指向`/proc/self/mountinfo`的链接。



### 如何获取文件系统的使用情况

从底层看来，使用的是`statfs`这个系统调用，这个系统调用可以获取上面说的所有信息，实现上考虑使用[这个库](https://docs.rs/nix/0.22.1/nix/all.html)。



有关`statfs`以及`statvfs`的区别，查看[这个链接](https://stackoverflow.com/questions/1653163/difference-between-statvfs-and-statfs-system-calls)



### 如何过滤挂载列表

注意到`df`的默认输出并不包括`/proc/self/mountinfo`中的全部挂载点，而是只显示了一部分。在GNU coreutils中，过滤的代码在`df.c`的`filter_mount_list`以及`get_dev`中。



阅读代码之后总结下过滤掉的挂载点。

默认情况下，过滤掉dummy的挂载类型，至于什么是dummy的挂载类型，见coreutils的`lib/mountinfo.c`中的`ME_DUMMY`相关宏。



`df`还提供额外的三个参数用于控制挂载点的过滤：

`-a`: 显示所有挂载，输出的数量与`/proc/self/mountinfo`中的一致

`-l`: 只显示本地挂载，不显示网络挂载

`-x`: 指定文件系统，只显示该文件系统类型的挂载




# Breadcrumbs
<img align="right" src="assets/logo.svg" width="200" align="right" alt="Breadcrumbs Logo">

> *Making symlinks intuitive and maintainable.*

Breadcrumbs offers a simpler way to move up in directory hierarchies by using symbolic links with easy-to-understand "breadcrumbs" markers. This replaces the confusing `../` sequences, making it easier to track and manage paths.

The repository also includes a ready-to-use script that showcases how this method works in practice.

## Concept

### Traditional Approach
Consider the following directory structure:

```
project/
│
├─── module1/
│    └─── sub1/
│         └─── file1.txt
│
└─── module2/
     └─── sub2/
```

With the usual method, to create a symlink inside `sub2` to `file1.txt`, we run:

```sh
ln -s ../../module1/sub1/file1.txt
```

**Problems**:
- **Obscurity**: Multiple `../` sequences can be hard to decipher, especially in deeper directory structures.
- **Maintenance**: When restructuring directories, updating numerous symlinks becomes tedious.
- **Brittleness**: Modifying the directory structure can not only break links, but may also unintentionally reference existing files, leading to unpredictable issues.

### Breadcrumbs Approach
Using Breadcrumbs, the "upward" path from `sub2` to the `project` directory becomes more intuitive, such as `.project.bc/module1/sub1/file1.txt`.

**How It Works**:
1. **Initialization**: A breadcrumb symlink named `.project.bc` is created inside the `project` directory, pointing to the `project` directory itself (`.`).
2. **Progression**: As we traverse towards `sub2`, at each step, another breadcrumb symlink `.project.bc` is created pointing to the parent directory's breadcrumb (`../.project.bc`).
3. **End Result**: By the time we reach `sub2`, we've established a trail of breadcrumb symlinks, guiding us from `sub2` back to the `project` directory seamlessly.

After executing breadcrumbs for the given example:

```
project/
│
├─── module1/
│    └─── sub1/
│         └─── file1.txt
│
├─── module2
│    │
│    ├─── sub2/
│    │    └─── .project.bc -> ../.project.bc
│    │
│    └─── .project.bc -> ../.project.bc
│
└─── .project.bc -> .
```

Then, to create a symlink inside `sub2` to `file1.txt`, execute:

```sh
ln -s .project.bc/module1/sub1/file1.txt
```

**Benefits**:
- **Clarity**: Instead of puzzling over multiple `../`, the `.project.bc/` breadcrumb explicitly indicates the journey up to the `project` directory.
- **Maintainability**: Breadcrumbs simplify symlink updates; adding or removing hierarchy levels often requires minimal to no breadcrumb adjustments.
- **Robustness**: When a significant structural change occurs, the breadcrumb symlink explicitly fails, preventing unintended file references.
- **Documentation**: The presence of breadcrumbs highlights inter-module file references, offering clear insights into file dependencies.

## Script

### Installation

Clone the repository, then `cp` the `breadcrumbs` script in the `bin` directory.

1. **Global Installation**:
   ```
   sudo cp breadcrumbs /usr/bin
   ```

2. **Local Installation**:
   First, ensure `~/.local/bin` is in your `PATH`. Then:
   ```
   cp breadcrumbs ~/.local/bin
   ```

### Usage

The script's usage is as follows:

```
breadcrumbs [-n name] [-s /path/to/source] [-t /path/to/target]
```

- `-n name`: Specify a custom breadcrumb name. If not set, the source directory name becomes the default.
- `-s /path/to/source`: The end of the breadcrumb trail. If not set, it defaults to the current directory.
- `-t /path/to/target`: The start of the breadcrumb trail. If not set, it defaults to the current directory.

**Example**:

The command

```
breadcrumbs -n my-breadcrumb -s ~/abc -t ~/abc/def/ghi
```

will result in creating a breadcrumb symlink named `.my-breadcrumb.bc` at `~/abc` and replicating it all the way down to `~/abc/def/ghi`, forming a breadcrumb trail.

**Removing Breadcrumbs**:

To delete breadcrumb symlinks, run:
```
rm **/.my-breadcrumb.bc
```
Execute this from the root breadcrumb directory or any relevant subdirectory.

**Note**: Linux distributions often limit symlink redirection to about 20. For deep directory trails, be aware of this constraint.

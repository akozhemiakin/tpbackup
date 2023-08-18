# Targetprocess Backup Utility

CLI utility that allows to backup
[Targetprocess](https://www.targetprocess.com/guide/) resources.

[![asciicast](https://asciinema.org/a/jbWenlKrO1t6R250k4lL3BBLF.svg)](https://asciinema.org/a/jbWenlKrO1t6R250k4lL3BBLF)

## Installation
You can install the utility with the following command if you have
`cargo` installed:
```
cargo install tpbackup
```

## Usage
This utility cycles through all or some of Targetprocess resources and backs
up each type of resource into a separate JSON file. It also provides
an option to package these JSON files into a single tarball.

**Backup to a folder:**
```bash
tpbackup backup -u user -p password example.tpondemand.com
```

**Backup to a tar.gz archive:**
```bash
tpbackup backup -u user -p password --compress example.tpondemand.com
```

**Backup a subset of the resources**
```bash
tpbackup backup -u user -p password -r UserStories,Bugs example.tpondemand.com
``````

**Output a list of default resources**
```bash
tpbackup resources
```

For additional configuration options, run:
```bash
tpbackup help backup
```
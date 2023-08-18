# Targetprocess Backup Utility

CLI utility that allows to backup
[Targetprocess](https://www.targetprocess.com/guide/) resources.

## Usage
This utility cycles through all or some of Targetprocess resources and backs
up each type of resource into a separate JSON file. It also provides
an option to package these JSON files into a single tarball.

**Backup to a folder:**
```bash
./tpbackup backup -u user -p password example.tpondemand.com
```

**Backup to a tar.gz archive:**
```bash
./tpbackup backup -u user -p password --compress example.tpondemand.com
```

**Backup a subset of the resources**
```bash
./tpbackup backup -u user -p password -r UserStories,Bugs example.tpondemand.com
``````

**Output a list of default resources**
```bash
./tpbackup resources
```

For additional configuration options, run:
```bash
./tpbackup help backup
```
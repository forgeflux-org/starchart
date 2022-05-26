# Published Crawled Data

Starchart publishes all crawled data. This document explains the
format(s) and the directory structure of the published data.

## Directory Structure

```bash
(lab)➜  starchart tree data
data
└── git.batsense.net
	├── instance.yml
	└── realaravinth
		├── analysis-of-captcha-systems
		│   └── publiccode.yml
        └── user.yml
```

> Snippet of data crawled from git.batsense.net

## Forge

Each forge instance gets its own directory in the repository root path
specified in the [configuration](../config/default.toml). All data
crawled from an instance will be stored in the instance's directory
only.

Each forge instance directory contains an `instance.yml` file that
describes the instance. The schema of `instance.yml` might change as
starchart is currently under development.

```yml
---
hostname: git.batsense.net
forge_type: gitea
```

> example instance.yml

## User

A forge instance's user gets their own subdirectory in starchart and an
`user.yml` to describe them. Information on all their repositories will be stored under
this subdirectory.

Like `instance.yml`, `user.yml` schema is not finalized too.

```yml
---
hostname: git.batsense.net
username: realaravinth
html_link: "https://git.batsense.net/realaravinth"
profile_photo: "https://git.batsense.net/avatars/bc11e95d9356ac4bdc035964be00ff0d"
```

> example user.yml

## Repository

Repository information is stored under the owner's subdirectory.
Currently, partial support for
[publiccodeyml](https://yml.publiccode.tools/) is implemented. So all
repository information is stored in `publiccode.yml` under the
repository subdirectory.

```yml
---
publiccodeYmlVersion: "0.2"
name: git.batsense.net
url: "https://git.batsense.net/realaravinth/git.batsense.net"
description:
    en:
        shortDescription: "Instance administration logs and discussions pertaining to this Gitea instance. Have a question about git.batsense.net? Please create an issue on this repository! :)"
```

> example publiccode.yml implemented by starchart

See
[forgeflux-org/starchart#3](https://github.com/forgeflux-org/starchart/issues/3) and
[publiccodeyml/publiccodeyml/discussions](https://github.com/publiccodeyml/publiccode.yml/discussions/157) for more information.

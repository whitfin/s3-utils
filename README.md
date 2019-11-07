# s3-utils
[![Crates.io](https://img.shields.io/crates/v/s3-utils.svg)](https://crates.io/crates/s3-utils) [![Build Status](https://img.shields.io/travis/whitfin/s3-utils.svg)](https://travis-ci.org/whitfin/s3-utils)

Utilities and tools based around Amazon S3 to provide convenience APIs in a CLI.

This tool contains a small set of command line utilities for working with Amazon S3, focused on including features which are not readily available in the S3 API. It has evolved from various scripts and use cases during work life, but packaged into something a little more useful. It's likely that more tools will be added over time as they become useful and/or required.

All S3 interaction is controlled by [rusoto_s3](https://crates.io/crates/rusoto_s3).

* [Installation](#installation)
* [Commands](#commands)
  + [concat](#concat)
  + [rename](#rename)
  + [report](#report)

## Installation

You can install `s3-utils` from either this repository, or from Crates (once it's published):

```shell
# install from Cargo
$ cargo install s3-utils

# install the latest from GitHub
$ cargo install --git https://github.com/whitfin/s3-utils.git
```

## Commands

Credentials can be configured by following the instructions on the [AWS Documentation](https://docs.aws.amazon.com/cli/latest/userguide/cli-environment.html). Almost every command you might use will take this shape:

```shell
$ AWS_ACCESS_KEY_ID=MY_ACCESS_KEY_ID \
    AWS_SECRET_ACCESS_KEY=MY_SECRET_ACCESS_KEY \
    AWS_DEFAULT_REGION=MY_AWS_REGION \
    s3-utils <subcommand> <arguments>
```

There are several switches available on almost all commands (such as `-d` to dry run an operation), but please check the command documentation before assuming it does exist. Each command exposes a `-h` switch to show a help menu, as standard. The examples below will omit the `AWS_` environment variables for brevity.

### concat

This command is focused around concatenation of files in S3. You can concatenate files in a basic manner just by providing a source pattern, and a target file path:

```shell
$ s3-utils concat my.bucket.name 'archives/*.gz' 'archive.gz'
```

If the case you're working with long paths, you can add a prefix on the bucket name to avoid having to type it all out multiple times. In the following case, `*.gz` and `archive.gz` are relative to the `my/annoyingly/nested/path/` prefix.

```shell
$ s3-utils concat my.bucket.name/my/annoyingly/nested/path/ '*.gz' 'archive.gz'
```

You can also use pattern matching (driven by the official `regex` crate), to use segments of the source paths in your target paths. Here is an example of mapping a date hierarchy (`YYYY/MM/DD`) to a flat structure (`YYYY-MM-DD`):

```shell
$ s3-utils concat my.bucket.name 'date-hierachy/(\d{4})/(\d{2})/(\d{2})/*.gz' 'flat-hierarchy/$1-$2-$3.gz'
```

In this case, all files in `2018/01/01/*` would be mapped to `2018-01-01.gz`. Don't forget to add single quotes around your expressions to avoid any pesky shell expansions!

In order to concatenate files remotely (i.e. without pulling them to your machine), this tool uses the Multipart Upload API of S3. This means that all limitations of that API are inherited by this tool. Usually, this isn't an issue, but one of the more noticeable problems is that files smaller than 5MB cannot be concatenated. To avoid wasted AWS calls, this is currently caught in the client layer and will result in a client side error. Due to the complexity in working around this, it's currently unsupported to join files with a size smaller than 5MB.

### rename

The `rename` command offers dynamic file renaming using patterns, without having to download files. The main utility in this command is being able to use patterns to rename large amounts of files in a single command.

You can rename files in a basic manner, such as simply changing their prefix:

```shell
$ s3-utils rename my.bucket.name 'my-directory/(.*)' 'my-new-directory/$1'
```

Although basic, this shows how you can use captured patterns in your renaming operations. This allows you to do much more complicated mappings, such as transforming an existing tree hierarchy into flat files:

```shell
$ s3-utils rename my.bucket.name '(.*)/(.*)/(.*)' '$1-$2-$3'
```

This is a very simple model, but provides a pretty flexible tool to change a lot of stuff pretty quickly.

Due to limitations in the current AWS S3 API, this command is unable to work with files larger than 5GB in size. At some point we may add a workaround for this, but for now this is likely to throw an error.

### report

Reports generate metadata about an S3 bucket or subdirectory thereof. They can be used to inspect things like file sizes, modification dates, etc. This command is extremely simple as it's fairly un-customizable:

```shell
$ s3-utils report my.bucket.name
$ s3-utils report my.bucket.name/my/directory/path
```

This generates shell output which follows a relatively simple format, meant to be easily extensible and (hopefully) convenient in shell pipelines. The general format is pretty stable, but certain formatting may change over time (spacing, number formatting, etc).

Below is an example based on a real S3 bucket (although with fake names):

```
[general]
total_time=7s
total_space=1.94TB
total_files=51,152

[file_size]
average_file_size=37.95MB
average_file_bytes=37949529
largest_file_size=1.82GB
largest_file_bytes=1818900684
largest_file_name=path/to/my_largest_file.txt.gz
smallest_file_size=54B
smallest_file_bytes=54
smallest_file_name=path/to/my_smallest_file.txt.gz
smallest_file_others=12

[extensions]
unique_extensions=1
most_frequent_extension=gz

[modification]
earliest_file_date=2016-06-11T17:36:57.000Z
earliest_file_name=path/to/my_earliest_file.txt.gz
earliest_file_others=3
latest_file_date=2017-01-01T00:03:19.000Z
latest_file_name=path/to/my_latest_file.txt.gz
```

This sample report is based on the initial builds of this subcommand, so depending on when you visit this tool there may be more (or less) included in the generated report.


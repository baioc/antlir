#!/usr/bin/env python3
# Copyright (c) Facebook, Inc. and its affiliates.
#
# This source code is licensed under the MIT license found in the
# LICENSE file in the root directory of this source tree.

import argparse

from jinja2 import Environment

parser = argparse.ArgumentParser()
parser.add_argument("template")
parser.add_argument("name")


def main():
    args = parser.parse_args()
    env = Environment(
        trim_blocks=True,
        lstrip_blocks=True,
    )
    with open(args.template) as tmpl_file:
        tmpl = tmpl_file.read()
        print(
            env.compile(
                tmpl,
                name=args.name,
                # generate python source, not bytecode
                raw=True,
                # generated code can be imported without environment global set
                defer_init=True,
            )
        )


if __name__ == "__main__":
    main()

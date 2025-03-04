#!/usr/bin/env python3
from sys import argv


def main(path):
    lines = []

    f = open(path, "r")
    read_lines = f.readlines()
    for line in read_lines:
        lines.append(line.strip())

    parsing_method = False

    methods = []
    method = ""
    comment = ""
    send = ""
    recv = ""

    for (idx, i) in enumerate(lines):
        if not i.startswith("//"):
            continue

        if i == ("// RPCAPI:"):
            parsing_method = True
            continue

        if parsing_method:
            if i.startswith("// --> "):
                method = i.split()[5][1:-2]
                recv = i[3:]
                continue

            if i.startswith("// <-- "):
                send = i[3:]
                parsing_method = False
                methods.append((method, comment.strip(), recv, send, idx + 2))
                comment = ""
                continue

            comment += i[3:] + "\n"

    print("\n## Methods")
    for i in methods:
        print(f"* [`{i[0]}`](#{i[0]})")

    print("\n")
    for i in methods:
        print(f"### `{i[0]}`\n")
        print(f"{i[1]}")
        ghlink = "%s%s%s%d" % (
            "https://github.com/darkrenaissance/darkfi/blob/master/",
            path.replace("../", ""),
            "#L",
            i[4],
        )
        print(f'<br><sup><a href="{ghlink}">[src]</a></sup>')
        print("\n```json")
        print(i[2])
        print(i[3])
        print("```")


if __name__ == "__main__":
    main(argv[1])

#!/usr/bin/env python3
"""Print Maven coordinates from a module pom.xml: groupId<TAB>artifactId<TAB>version (no newline in fields)."""

import sys
import xml.etree.ElementTree as ET
from typing import Optional

NS = "http://maven.apache.org/POM/4.0.0"


def qn(local: str) -> str:
    return f"{{{NS}}}{local}"


def direct_text(root: ET.Element, local: str) -> Optional[str]:
    el = root.find(qn(local))
    if el is None or el.text is None:
        return None
    t = el.text.strip()
    return t or None


def main() -> None:
    if len(sys.argv) != 2:
        print("usage: read-maven-pom-coords.py <pom.xml>", file=sys.stderr)
        sys.exit(2)
    root = ET.parse(sys.argv[1]).getroot()
    gid = direct_text(root, "groupId")
    aid = direct_text(root, "artifactId")
    ver = direct_text(root, "version")
    parent = root.find(qn("parent"))
    if parent is not None:
        if gid is None:
            el = parent.find(qn("groupId"))
            if el is not None and el.text:
                gid = el.text.strip()
        if ver is None:
            el = parent.find(qn("version"))
            if el is not None and el.text:
                ver = el.text.strip()
    if not gid or not aid or not ver:
        print("missing groupId, artifactId or version in POM", file=sys.stderr)
        sys.exit(1)
    print(f"{gid}\t{aid}\t{ver}")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Verificações da documentação do OSMarcio Linux.

- Links relativos em arquivos .md apontam para arquivos existentes.
- Arquivos .drawio são XML válido, com ao menos uma aba (<diagram>) e
  cada aba contém células (mxCell).
Uso: docs-check.py [--diagrams-only]
"""
import pathlib
import re
import sys
import xml.etree.ElementTree as ET

ROOT = pathlib.Path(__file__).resolve().parents[2]
SKIP_DIRS = {"out", "target", ".git", "images"}
LINK = re.compile(r"\[[^\]]*\]\(([^)\s]+)\)")


def md_files():
    for p in ROOT.rglob("*.md"):
        if not SKIP_DIRS.intersection(p.relative_to(ROOT).parts):
            yield p


def check_links():
    errors = []
    for md in md_files():
        text = md.read_text(encoding="utf-8")
        for target in LINK.findall(text):
            if re.match(r"^[a-z]+:", target) or target.startswith("#"):
                continue  # URL externa ou âncora
            path = target.split("#", 1)[0]
            if not path:
                continue
            if not (md.parent / path).exists():
                errors.append(f"{md.relative_to(ROOT)}: link quebrado -> {target}")
    return errors


def check_diagrams():
    errors, count = [], 0
    for f in sorted((ROOT / "diagrams" / "drawio").glob("*.drawio")):
        try:
            tree = ET.parse(f)
        except ET.ParseError as e:
            errors.append(f"{f.relative_to(ROOT)}: XML inválido: {e}")
            continue
        pages = tree.getroot().findall("diagram")
        if not pages:
            errors.append(f"{f.relative_to(ROOT)}: nenhuma aba <diagram>")
        for d in pages:
            cells = d.findall(".//mxCell")
            if len(cells) < 3:
                errors.append(f"{f.relative_to(ROOT)}: aba '{d.get('name')}' vazia")
        count += len(pages)
    print(f"diagramas: {count} aba(s) verificadas")
    return errors


def main():
    errors = check_diagrams()
    if "--diagrams-only" not in sys.argv:
        link_errors = check_links()
        print(f"markdown: {sum(1 for _ in md_files())} arquivo(s) verificados")
        errors += link_errors
    for e in errors:
        print("ERRO", e)
    sys.exit(1 if errors else 0)


if __name__ == "__main__":
    main()

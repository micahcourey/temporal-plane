#!/usr/bin/env python3
"""
parse-csv.py — Convert CSV/TSV reference files to JSONL context files

Reads a CSV or TSV file (typically a database extract) and converts it to
JSONL format with optional column mapping, filtering, and categorization.

Usage:
    # Basic: convert CSV to JSONL with automatic column name lowercasing
    python3 setup/scripts/parse-csv.py reference/roles.csv --output .ai/context/roles.jsonl

    # With column mapping: rename CSV columns to JSONL field names
    python3 setup/scripts/parse-csv.py reference/PRVLG_TYPE.csv \
        --output .ai/context/permissions.jsonl \
        --map PRVLG_TYPE_CD=permission PRVLG_TYPE_DESC=description PRVLG_TYPE_GRP_CD=group

    # With a categorizer: auto-categorize records by prefix patterns
    python3 setup/scripts/parse-csv.py reference/PRVLG_TYPE.csv \
        --output .ai/context/permissions.jsonl \
        --map PRVLG_TYPE_CD=permission PRVLG_TYPE_DESC=description \
        --categorize permission

    # TSV input
    python3 setup/scripts/parse-csv.py reference/data.tsv --delimiter tab --output output.jsonl

    # Preview without writing (prints to stdout)
    python3 setup/scripts/parse-csv.py reference/roles.csv
"""

import argparse
import csv
import json
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple


# Common prefix → category mappings for auto-categorization
PREFIX_CATEGORIES = {
    # Permission/privilege prefixes
    "ACCESS_": "page_access",
    "VIEW_": "read",
    "LIST_": "read",
    "GET_": "read",
    "READ_": "read",
    "ADD_": "create",
    "CREATE_": "create",
    "NEW_": "create",
    "EDIT_": "update",
    "UPDATE_": "update",
    "MODIFY_": "update",
    "DELETE_": "delete",
    "REMOVE_": "delete",
    "ARCHIVE_": "delete",
    "DOWNLOAD_": "file_ops",
    "UPLOAD_": "file_ops",
    "EXPORT_": "file_ops",
    "IMPORT_": "file_ops",
    "APPROVE_": "approval",
    "REJECT_": "approval",
    "ADJUDICATE_": "approval",
    "SUBMIT_": "workflow",
    "ASSIGN_": "workflow",
    "MANAGE_": "admin",
    "ADMIN_": "admin",
    "CONFIG_": "admin",
}


def parse_column_mappings(mapping_args: List[str]) -> Dict[str, str]:
    """Parse column mapping arguments like 'SOURCE_COL=target_field'."""
    mappings = {}
    for arg in mapping_args:
        if "=" not in arg:
            print(f"Warning: ignoring invalid mapping '{arg}' (expected SOURCE=target)", file=sys.stderr)
            continue
        source, target = arg.split("=", 1)
        mappings[source.strip()] = target.strip()
    return mappings


def categorize_value(value: str) -> str:
    """Auto-categorize a value based on its prefix."""
    upper = value.upper()
    for prefix, category in PREFIX_CATEGORIES.items():
        if upper.startswith(prefix):
            return category
    return "other"


def transform_row(
    row: Dict[str, str],
    column_mappings: Dict[str, str],
    categorize_field: Optional[str],
    drop_empty: bool,
) -> Dict[str, Any]:
    """Transform a CSV row into a JSONL record."""
    record = {}

    for csv_col, value in row.items():
        csv_col = csv_col.strip()
        value = value.strip() if value else ""

        # Skip empty values if requested
        if drop_empty and not value:
            continue

        # Apply column mapping or lowercase the column name
        if column_mappings and csv_col in column_mappings:
            field_name = column_mappings[csv_col]
        elif column_mappings:
            # If mappings are specified, skip unmapped columns
            continue
        else:
            # Default: lowercase and clean the column name
            field_name = re.sub(r'[^a-z0-9_]', '_', csv_col.lower())
            field_name = re.sub(r'_+', '_', field_name).strip('_')

        if value:
            record[field_name] = value

    # Auto-categorize if requested
    if categorize_field and categorize_field in record:
        record["category"] = categorize_value(record[categorize_field])

    return record


def detect_delimiter(file_path: Path) -> str:
    """Auto-detect CSV delimiter by reading the first line."""
    with open(file_path, "r", encoding="utf-8-sig") as f:
        first_line = f.readline()

    # Count occurrences of common delimiters
    tab_count = first_line.count("\t")
    comma_count = first_line.count(",")
    pipe_count = first_line.count("|")
    semicolon_count = first_line.count(";")

    counts = {
        "\t": tab_count,
        ",": comma_count,
        "|": pipe_count,
        ";": semicolon_count,
    }

    # Return the most common delimiter
    return max(counts, key=counts.get) if max(counts.values()) > 0 else ","


def main():
    parser = argparse.ArgumentParser(
        description="Convert CSV/TSV reference files to JSONL context files"
    )
    parser.add_argument("input", help="Path to the CSV/TSV input file")
    parser.add_argument(
        "--output", "-o",
        help="Output JSONL file path (default: stdout)",
        default=None,
    )
    parser.add_argument(
        "--map", "-m",
        nargs="*",
        help="Column mappings: SOURCE_COL=target_field (e.g., PRVLG_TYPE_CD=permission)",
        default=[],
    )
    parser.add_argument(
        "--categorize", "-c",
        help="Auto-categorize records based on prefix patterns of this field",
        default=None,
    )
    parser.add_argument(
        "--delimiter", "-d",
        help="CSV delimiter: comma, tab, pipe, semicolon, or auto (default: auto)",
        default="auto",
    )
    parser.add_argument(
        "--drop-empty",
        action="store_true",
        help="Drop fields with empty values from output records",
    )
    parser.add_argument(
        "--filter",
        help="Only include rows where this field is non-empty (e.g., --filter ACTIVE_FLAG)",
        default=None,
    )

    args = parser.parse_args()
    input_path = Path(args.input)

    if not input_path.exists():
        print(f"Error: {input_path} does not exist", file=sys.stderr)
        sys.exit(1)

    # Resolve delimiter
    delimiter_map = {"comma": ",", "tab": "\t", "pipe": "|", "semicolon": ";"}
    if args.delimiter == "auto":
        delimiter = detect_delimiter(input_path)
    elif args.delimiter in delimiter_map:
        delimiter = delimiter_map[args.delimiter]
    else:
        delimiter = args.delimiter

    # Parse column mappings
    column_mappings = parse_column_mappings(args.map) if args.map else {}

    # Read and transform
    records = []
    with open(input_path, "r", encoding="utf-8-sig") as f:
        reader = csv.DictReader(f, delimiter=delimiter)
        for row in reader:
            # Apply filter if specified
            if args.filter:
                filter_val = row.get(args.filter, "").strip()
                if not filter_val:
                    continue

            record = transform_row(row, column_mappings, args.categorize, args.drop_empty)
            if record:  # Skip empty records
                records.append(record)

    # Build JSONL output
    lines = [json.dumps(record, ensure_ascii=False) for record in records]
    output = "\n".join(lines) + "\n" if lines else ""

    if args.output:
        output_path = Path(args.output)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, "w") as f:
            f.write(output)
        print(f"Wrote {len(records)} records to {output_path}")
    else:
        sys.stdout.write(output)
        print(f"\n# {len(records)} records parsed", file=sys.stderr)


if __name__ == "__main__":
    main()

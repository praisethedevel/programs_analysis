#!/usr/bin/env python3
# SPDX-License-Identifier: PACARL-1.0
# Copyright (c) 2026 praisethedevel <xsolutionx@gmail.com>. All Rights Reserved.
# Governed by the Praisethedevel Universal Architecture, Compulsory Attribution & Commercial Royalty License.
# Non-commercial / evaluation use only. Commercial exploitation strictly requires prior written agreement.
import os
import sys
import glob
import json
import struct
import subprocess
import re

WORKDIR = "/home/joe/programs_analysis"
SOLDECOMP = "/home/joe/soldecomp/soldecomp.py"

def run_cmd(cmd, cwd=WORKDIR):
    res = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    return res.stdout, res.stderr, res.returncode

def parse_elf_headers(filepath):
    # Use readelf or llvm-objdump
    out, _, _ = run_cmd(["readelf", "-h", "-S", "-s", "--dyn-syms", filepath])
    return out

def get_strings(filepath, min_len=4):
    out, _, _ = run_cmd(["strings", "-n", str(min_len), filepath])
    return [s.strip() for s in out.splitlines() if s.strip()]

def get_objdump_dump(filepath):
    dump_path = filepath.replace(".so", ".dump")
    if not os.path.exists(dump_path) or os.path.getsize(dump_path) == 0:
        print(f"Generating dump for {filepath}...")
        run_cmd(["llvm-objdump", "-S", "-d", "--print-imm-hex", "--no-show-raw-insn", filepath], cwd=WORKDIR)
        # write to dump_path
        res = subprocess.run(["llvm-objdump", "-S", "-d", "--print-imm-hex", "--no-show-raw-insn", filepath],
                             stdout=open(dump_path, "w"), stderr=subprocess.DEVNULL)
    return dump_path

def analyze_binary(filepath):
    base = os.path.basename(filepath)
    prog_key = base.replace(".so", "")
    size = os.path.getsize(filepath)
    
    print(f"=== Auditing {base} ({size} bytes) ===")
    
    # 1. soldecomp analyze
    soldecomp_out = os.path.join(WORKDIR, f"soldecomp_{prog_key}")
    os.makedirs(soldecomp_out, exist_ok=True)
    run_cmd(["python3", SOLDECOMP, "analyze", filepath, "--out", soldecomp_out])
    
    analysis_json_file = os.path.join(soldecomp_out, "analysis.json")
    soldecomp_info = {}
    if os.path.exists(analysis_json_file):
        try:
            with open(analysis_json_file, "r") as f:
                soldecomp_info = json.load(f)
        except Exception as e:
            soldecomp_info = {"error": str(e)}

    # 2. soldecomp detect-framework
    det_out, _, _ = run_cmd(["python3", SOLDECOMP, "detect-framework", filepath])
    framework_info = {}
    try:
        framework_info = json.loads(det_out)
    except Exception:
        framework_info = {"raw": det_out}

    # 3. Dynamic symbols & Syscalls
    syms_out, _, _ = run_cmd(["readelf", "--dyn-syms", filepath])
    syscalls = []
    exported_symbols = []
    for line in syms_out.splitlines():
        parts = line.split()
        if len(parts) >= 8:
            sym_name = parts[-1]
            if "sol_" in sym_name:
                syscalls.append(sym_name)
            elif parts[3] == "FUNC" and parts[6] != "UND":
                exported_symbols.append(sym_name)

    # 4. Strings & instruction hints
    strs = get_strings(filepath)
    
    instruction_names = []
    error_messages = []
    for s in strs:
        if any(kw in s for kw in ["Instruction:", "Instruction::", "Error:", "Panicked", "panicked"]):
            if "Instruction:" in s or "Instruction::" in s:
                instruction_names.append(s)
            if "Error:" in s or "panicked" in s.lower():
                error_messages.append(s)

    # 5. Objdump dump and line count
    dump_file = get_objdump_dump(filepath)
    dump_lines = 0
    if os.path.exists(dump_file):
        with open(dump_file, "r", errors="ignore") as f:
            dump_lines = sum(1 for _ in f)

    report = {
        "binary": base,
        "program_key": prog_key,
        "size_bytes": size,
        "entrypoint": soldecomp_info.get("entrypoint"),
        "instruction_count": soldecomp_info.get("instruction_count"),
        "function_count": soldecomp_info.get("function_count"),
        "text_bytes": soldecomp_info.get("text_bytes"),
        "sbpf_version": soldecomp_info.get("sbpf", {}).get("version"),
        "framework": framework_info.get("framework"),
        "signals": framework_info.get("signals"),
        "syscalls": sorted(list(set(syscalls))),
        "exported_symbols": exported_symbols,
        "instructions_detected": sorted(list(set(instruction_names))),
        "sample_errors": error_messages[:15],
        "dump_file": os.path.basename(dump_file),
        "dump_lines": dump_lines,
    }
    return report

def main():
    binaries = sorted(glob.glob(os.path.join(WORKDIR, "*.so")))
    all_reports = {}
    for b in binaries:
        rep = analyze_binary(b)
        all_reports[rep["program_key"]] = rep
        
    summary_path = os.path.join(WORKDIR, "audit_summary.json")
    with open(summary_path, "w") as f:
        json.dump(all_reports, f, indent=2)
    print(f"\n[✓] Finished analysis. Summary saved to {summary_path}")

if __name__ == "__main__":
    main()

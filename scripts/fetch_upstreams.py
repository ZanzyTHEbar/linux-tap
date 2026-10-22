#!/usr/bin/env python3
"""Fetch explicit locked stepping stones without executing their build scripts."""
import argparse
import json
import subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]

def main():
    entries=json.loads((ROOT/'deps/upstreams.lock.json').read_text())['upstreams']
    by_name={entry['name']:entry for entry in entries}
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('names',nargs='+',choices=sorted(by_name))
    args=parser.parse_args()
    for name in args.names:
        entry=by_name[name]
        target=ROOT/'vendor'/name
        if target.exists():
            head=subprocess.check_output(['git','-C',str(target),'rev-parse','HEAD'],text=True).strip()
            assert head==entry['rev'],f'{name} has another revision; preserve local work and inspect it'
            print(f'{name}: existing revision matches; no checkout or deletion performed')
            continue
        target.mkdir(parents=True)
        subprocess.run(['git','init',str(target)],check=True,capture_output=True)
        subprocess.run(['git','-C',str(target),'remote','add','origin',entry['url']],check=True)
        subprocess.run(['git','-C',str(target),'fetch','--depth','1','origin',entry['rev']],check=True)
        subprocess.run(['git','-C',str(target),'checkout','--detach',entry['rev']],check=True)
        head=subprocess.check_output(['git','-C',str(target),'rev-parse','HEAD'],text=True).strip()
        assert head==entry['rev']
        print(f'{name}: verified {head}; not built or installed')

if __name__=='__main__':
    main()

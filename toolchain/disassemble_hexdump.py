#!/usr/bin/env python3

import struct
import os
import tempfile

list = [0x55, 0x48, 0x89, 0xe5, 0xb8, 0xd2, 0x4, 0x0, 0x0, 0x5d, 0xc3, 0x55, 0x48, 0x89, 0xe5, 0x41, 0x57, 0x48, 0x83, 0xec, 0x8, 0x48, 0x89, 0xbc, 0x24, 0x0, 0x0, 0x0, 0x0, 0x49, 0x89, 0xff, 0x4c, 0x89, 0xff, 0xe8, 0xd8, 0xff, 0xff, 0xff, 0x4c, 0x8b, 0xbc, 0x24, 0x0, 0x0, 0x0, 0x0, 0x49, 0x8b, 0x4f, 0x8, 0x4c, 0x89, 0xff, 0x40, 0x89, 0xc6, 0xff, 0xd1, 0x48, 0x83, 0xc4, 0x8, 0x41, 0x5f, 0x5d, 0xc3]

out = tempfile.NamedTemporaryFile(mode = 'wb')
out.write(struct.pack(f'{len(list)}B', *list))
out.flush()
os.system(f'objdump -b binary -D -m i386:x86-64 {out.name}')
out.close()

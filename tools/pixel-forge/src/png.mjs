import { deflateSync } from 'node:zlib';

const CRC_TABLE = new Uint32Array(256);
for (let n = 0; n < 256; n += 1) {
  let c = n;
  for (let k = 0; k < 8; k += 1) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  CRC_TABLE[n] = c >>> 0;
}

function crc32(buffer) {
  let c = 0xffffffff;
  for (let i = 0; i < buffer.length; i += 1) c = CRC_TABLE[(c ^ buffer[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const typeBuffer = Buffer.from(type, 'ascii');
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length, 0);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuffer, data])), 0);
  return Buffer.concat([length, typeBuffer, data, crc]);
}

export function encodePng(canvas) {
  const { w, h, data } = canvas;
  const stride = w * 4 + 1;
  const raw = Buffer.alloc(stride * h);
  for (let y = 0; y < h; y += 1) {
    raw[y * stride] = 0;
    data.copy(raw, y * stride + 1, y * w * 4, (y + 1) * w * 4);
  }
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0);
  ihdr.writeUInt32BE(h, 4);
  ihdr[8] = 8; ihdr[9] = 6; ihdr[10] = 0; ihdr[11] = 0; ihdr[12] = 0;
  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const idat = deflateSync(raw, { level: 9 });
  return Buffer.concat([signature, chunk('IHDR', ihdr), chunk('IDAT', idat), chunk('IEND', Buffer.alloc(0))]);
}

const colorCache = new Map();
export function parseColor(color) {
  if (Array.isArray(color)) return color;
  const cached = colorCache.get(color);
  if (cached) return cached;
  let hex = color.replace('#', '');
  if (hex.length === 3) hex = hex.split('').map((c) => c + c).join('');
  const value = [
    parseInt(hex.slice(0, 2), 16),
    parseInt(hex.slice(2, 4), 16),
    parseInt(hex.slice(4, 6), 16),
    hex.length >= 8 ? parseInt(hex.slice(6, 8), 16) : 255,
  ];
  colorCache.set(color, value);
  return value;
}

export class Canvas {
  constructor(w, h) {
    this.w = w; this.h = h; this.data = Buffer.alloc(w * h * 4);
  }
  set(x, y, color) {
    x = Math.round(x); y = Math.round(y);
    if (x < 0 || y < 0 || x >= this.w || y >= this.h) return;
    const [r, g, b, a] = parseColor(color);
    const i = (y * this.w + x) * 4;
    if (a >= 255) { this.data[i]=r; this.data[i+1]=g; this.data[i+2]=b; this.data[i+3]=255; return; }
    if (a <= 0) return;
    const sa = a / 255, da = this.data[i+3] / 255, oa = sa + da * (1-sa);
    if (oa <= 0) return;
    this.data[i] = Math.round((r*sa + this.data[i]*da*(1-sa))/oa);
    this.data[i+1] = Math.round((g*sa + this.data[i+1]*da*(1-sa))/oa);
    this.data[i+2] = Math.round((b*sa + this.data[i+2]*da*(1-sa))/oa);
    this.data[i+3] = Math.round(oa*255);
  }
  rect(x, y, w, h, color) {
    for (let j = 0; j < h; j += 1) for (let i = 0; i < w; i += 1) this.set(x+i, y+j, color);
  }
  blit(src, dx = 0, dy = 0) {
    for (let y = 0; y < src.h; y += 1) for (let x = 0; x < src.w; x += 1) {
      const i = (y * src.w + x) * 4;
      this.set(dx + x, dy + y, [src.data[i], src.data[i+1], src.data[i+2], src.data[i+3]]);
    }
  }
}

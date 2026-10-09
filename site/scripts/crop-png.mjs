#!/usr/bin/env node
/**
 * Crop a PNG. No dependencies — Node's `zlib` is enough, and a dependency here
 * would mean a lockfile entry that exists only to trim a screenshot.
 *
 * Exists because headless Chrome does not reliably paint an SVG all the way to
 * the bottom of a `--window-size` viewport: on Chrome 142 the painted area stops
 * around 544 px of a 630 px window, so anything drawn below that is silently
 * missing from the screenshot while the background behind it still lands. The
 * fix is not to design around a magic number that changes between Chrome
 * versions — it is to render with generous headroom and crop the result.
 *
 * Usage:
 *   node scripts/crop-png.mjs <in.png> <out.png> <width> <height>
 *
 * The crop is always the top-left `width`x`height` region. Only 8-bit
 * truecolour and truecolour-with-alpha PNGs are accepted; anything else is
 * rejected loudly rather than silently mangled.
 */

import { readFileSync, writeFileSync } from "node:fs";
import { inflateSync, deflateSync } from "node:zlib";

const [, , input, output, widthArg, heightArg] = process.argv;
const width = Number(widthArg);
const height = Number(heightArg);

if (!input || !output || !Number.isInteger(width) || !Number.isInteger(height)) {
  console.error("usage: crop-png.mjs <in.png> <out.png> <width> <height>");
  process.exit(2);
}

const buf = readFileSync(input);

// --- read -----------------------------------------------------------------

const PNG_SIGNATURE = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];
for (let i = 0; i < PNG_SIGNATURE.length; i++) {
  if (buf[i] !== PNG_SIGNATURE[i]) throw new Error(`${input} is not a PNG`);
}

let pos = 8;
let ihdr = null;
const idat = [];
while (pos < buf.length) {
  const length = buf.readUInt32BE(pos);
  const type = buf.toString("ascii", pos + 4, pos + 8);
  const data = buf.subarray(pos + 8, pos + 8 + length);
  if (type === "IHDR") {
    ihdr = {
      width: data.readUInt32BE(0),
      height: data.readUInt32BE(4),
      depth: data[8],
      colourType: data[9],
      compression: data[10],
      filter: data[11],
      interlace: data[12],
    };
  } else if (type === "IDAT") {
    idat.push(data);
  } else if (type === "IEND") {
    break;
  }
  pos += 12 + length; // length + type + data + crc
}

if (!ihdr) throw new Error(`${input} has no IHDR`);
if (ihdr.depth !== 8 || (ihdr.colourType !== 2 && ihdr.colourType !== 6)) {
  throw new Error(
    `unsupported PNG: depth ${ihdr.depth}, colour type ${ihdr.colourType}. ` +
      "crop-png.mjs handles 8-bit truecolour (2) and truecolour+alpha (6) only.",
  );
}
if (ihdr.interlace !== 0) {
  throw new Error("interlaced PNGs are not supported");
}
if (width > ihdr.width || height > ihdr.height) {
  throw new Error(
    `cannot crop ${width}x${height} out of ${ihdr.width}x${ihdr.height}: the source is smaller`,
  );
}

const channels = ihdr.colourType === 6 ? 4 : 3;
const stride = ihdr.width * channels;
const raw = inflateSync(Buffer.concat(idat));
if (raw.length < (stride + 1) * ihdr.height) {
  throw new Error("truncated PNG data");
}

// --- unfilter -------------------------------------------------------------

/** Reverse one PNG scanline filter, in place. */
function unfilter(type, line, previous, bpp) {
  switch (type) {
    case 0:
      return line;
    case 1:
      for (let x = bpp; x < line.length; x++) line[x] = (line[x] + line[x - bpp]) & 0xff;
      return line;
    case 2:
      for (let x = 0; x < line.length; x++) line[x] = (line[x] + previous[x]) & 0xff;
      return line;
    case 3:
      for (let x = 0; x < line.length; x++) {
        const a = x >= bpp ? line[x - bpp] : 0;
        line[x] = (line[x] + ((a + previous[x]) >> 1)) & 0xff;
      }
      return line;
    case 4:
      for (let x = 0; x < line.length; x++) {
        const a = x >= bpp ? line[x - bpp] : 0;
        const b = previous[x];
        const c = x >= bpp ? previous[x - bpp] : 0;
        const p = a + b - c;
        const pa = Math.abs(p - a);
        const pb = Math.abs(p - b);
        const pc = Math.abs(p - c);
        const pr = pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
        line[x] = (line[x] + pr) & 0xff;
      }
      return line;
    default:
      throw new Error(`unknown PNG filter type ${type}`);
  }
}

const pixels = Buffer.alloc(stride * ihdr.height);
{
  let previous = Buffer.alloc(stride);
  for (let y = 0; y < ihdr.height; y++) {
    const offset = y * (stride + 1);
    const type = raw[offset];
    const line = unfilter(type, Buffer.from(raw.subarray(offset + 1, offset + 1 + stride)), previous, channels);
    line.copy(pixels, y * stride);
    previous = line;
  }
}

// --- crop -----------------------------------------------------------------

const outStride = width * channels;
const out = Buffer.alloc((outStride + 1) * height);
for (let y = 0; y < height; y++) {
  out[y * (outStride + 1)] = 0; // filter: none, so the output is reproducible
  pixels.copy(
    out,
    y * (outStride + 1) + 1,
    y * stride,
    y * stride + outStride,
  );
}

// --- write ----------------------------------------------------------------

const CRC_TABLE = (() => {
  const table = new Int32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c;
  }
  return table;
})();

function crc32(bytes) {
  let c = 0xffffffff;
  for (const byte of bytes) c = CRC_TABLE[(c ^ byte) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const body = Buffer.concat([Buffer.from(type, "ascii"), data]);
  const out = Buffer.alloc(body.length + 8);
  out.writeUInt32BE(data.length, 0);
  body.copy(out, 4);
  out.writeUInt32BE(crc32(body), body.length + 4);
  return out;
}

const ihdrOut = Buffer.alloc(13);
ihdrOut.writeUInt32BE(width, 0);
ihdrOut.writeUInt32BE(height, 4);
ihdrOut[8] = 8;
ihdrOut[9] = ihdr.colourType;
ihdrOut[10] = 0;
ihdrOut[11] = 0;
ihdrOut[12] = 0;

writeFileSync(
  output,
  Buffer.concat([
    Buffer.from(PNG_SIGNATURE),
    chunk("IHDR", ihdrOut),
    chunk("IDAT", deflateSync(out, { level: 9 })),
    chunk("IEND", Buffer.alloc(0)),
  ]),
);

console.log(`crop-png: ${input} (${ihdr.width}x${ihdr.height}) -> ${output} (${width}x${height})`);
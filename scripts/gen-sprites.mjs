#!/usr/bin/env node
// Gera os sprites originais do EnzoBallZ em PNG, sem dependências externas.
//
// O script é determinístico: rodar de novo produz exatamente os mesmos bytes.
// Os PNGs gerados ficam em assets/sprites/ e são embutidos no cliente WASM via
// include_bytes!, então não há carregamento assíncrono de assets no navegador.
//
// Uso: node scripts/gen-sprites.mjs

import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const OUT = join(ROOT, "assets", "sprites");

// ---------------------------------------------------------------------------
// Codificador PNG (RGBA 8 bits, sem filtro por linha)
// ---------------------------------------------------------------------------

const CRC_TABLE = new Uint32Array(256);
for (let n = 0; n < 256; n += 1) {
  let c = n;
  for (let k = 0; k < 8; k += 1) {
    c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  }
  CRC_TABLE[n] = c >>> 0;
}

function crc32(buffer) {
  let c = 0xffffffff;
  for (let i = 0; i < buffer.length; i += 1) {
    c = CRC_TABLE[(c ^ buffer[i]) & 0xff] ^ (c >>> 8);
  }
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const typeBuffer = Buffer.from(type, "ascii");
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length, 0);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuffer, data])), 0);
  return Buffer.concat([length, typeBuffer, data, crc]);
}

function encodePng(canvas) {
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
  ihdr[8] = 8; // profundidade
  ihdr[9] = 6; // RGBA
  ihdr[10] = 0; // compressão
  ihdr[11] = 0; // filtro
  ihdr[12] = 0; // sem entrelaçamento

  const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const idat = deflateSync(raw, { level: 9 });
  return Buffer.concat([
    signature,
    chunk("IHDR", ihdr),
    chunk("IDAT", idat),
    chunk("IEND", Buffer.alloc(0)),
  ]);
}

// ---------------------------------------------------------------------------
// Paleta e canvas
// ---------------------------------------------------------------------------

const P = {
  outline: "#1b1220",
  outlineSoft: "#2c2036",
  hair: "#26324a",
  hairLight: "#46618c",
  skin: "#f2b48c",
  skinShade: "#d18a63",
  eye: "#241a2e",
  white: "#f8f4e8",
  hoodie: "#f28c28",
  hoodieDark: "#c46118",
  hoodieLight: "#ffb347",
  pants: "#2f4b7c",
  pantsDark: "#22365a",
  shoe: "#e8483f",
  shoeLight: "#ff8074",
  aura: "#5ef2ff",
  auraLight: "#c8ffff",
  auraDeep: "#3a7bd5",
  slime: "#58c45a",
  slimeLight: "#8fe07a",
  slimeDark: "#3d9a45",
  slimeHurt: "#a9d99a",
  dirt: "#8a5a36",
  dirtDark: "#6b4226",
  dirtLight: "#a9744a",
  grass: "#5fbf4a",
  grassLight: "#8fe07a",
  grassDark: "#3f8f38",
  stone: "#7d7f94",
  stoneLight: "#a9abbd",
  stoneDark: "#56576b",
  wood: "#a9744a",
  woodLight: "#c99563",
  woodDark: "#744a2a",
  gold: "#f6c445",
  goldLight: "#ffe38a",
  goldDark: "#b07a1e",
  steel: "#c7d3e3",
  steelDark: "#7d8ba3",
  gem: "#b06bff",
  gemLight: "#e0b8ff",
  potion: "#e8483f",
  potionLight: "#ff8a7a",
  cloud: "#e8ecff",
  cloudShade: "#c7cdf0",
};

const colorCache = new Map();

function parseColor(color) {
  if (Array.isArray(color)) return color;
  const cached = colorCache.get(color);
  if (cached) return cached;

  let hex = color.replace("#", "");
  if (hex.length === 3) {
    hex = hex
      .split("")
      .map((char) => char + char)
      .join("");
  }
  const value = [
    parseInt(hex.slice(0, 2), 16),
    parseInt(hex.slice(2, 4), 16),
    parseInt(hex.slice(4, 6), 16),
    hex.length >= 8 ? parseInt(hex.slice(6, 8), 16) : 255,
  ];
  colorCache.set(color, value);
  return value;
}

class Canvas {
  constructor(w, h) {
    this.w = w;
    this.h = h;
    this.data = Buffer.alloc(w * h * 4);
  }

  set(x, y, color) {
    x = Math.round(x);
    y = Math.round(y);
    if (x < 0 || y < 0 || x >= this.w || y >= this.h) return;

    const [r, g, b, a] = parseColor(color);
    const index = (y * this.w + x) * 4;
    if (a >= 255) {
      this.data[index] = r;
      this.data[index + 1] = g;
      this.data[index + 2] = b;
      this.data[index + 3] = 255;
      return;
    }
    if (a <= 0) return;

    // Alpha blending "source over" para brilhos e partículas.
    const sa = a / 255;
    const da = this.data[index + 3] / 255;
    const oa = sa + da * (1 - sa);
    if (oa <= 0) return;
    this.data[index] = Math.round(
      (r * sa + this.data[index] * da * (1 - sa)) / oa,
    );
    this.data[index + 1] = Math.round(
      (g * sa + this.data[index + 1] * da * (1 - sa)) / oa,
    );
    this.data[index + 2] = Math.round(
      (b * sa + this.data[index + 2] * da * (1 - sa)) / oa,
    );
    this.data[index + 3] = Math.round(oa * 255);
  }

  rect(x, y, w, h, color) {
    for (let j = 0; j < h; j += 1) {
      for (let i = 0; i < w; i += 1) {
        this.set(x + i, y + j, color);
      }
    }
  }

  ellipse(cx, cy, rx, ry, color) {
    const ix = Math.max(1, Math.ceil(rx));
    const iy = Math.max(1, Math.ceil(ry));
    for (let y = -iy; y <= iy; y += 1) {
      for (let x = -ix; x <= ix; x += 1) {
        const nx = x / Math.max(rx, 0.5);
        const ny = y / Math.max(ry, 0.5);
        if (nx * nx + ny * ny <= 1) {
          this.set(cx + x, cy + y, color);
        }
      }
    }
  }

  circle(cx, cy, r, color) {
    this.ellipse(cx, cy, r, r, color);
  }

  line(x0, y0, x1, y1, color, thickness = 1) {
    const steps = Math.max(1, Math.ceil(Math.hypot(x1 - x0, y1 - y0)));
    for (let i = 0; i <= steps; i += 1) {
      const t = i / steps;
      this.circle(x0 + (x1 - x0) * t, y0 + (y1 - y0) * t, thickness / 2, color);
    }
  }

  arc(cx, cy, radius, start, end, thickness, color) {
    const steps = Math.max(10, Math.ceil(Math.abs(end - start) * radius));
    for (let i = 0; i <= steps; i += 1) {
      const angle = start + ((end - start) * i) / steps;
      this.circle(
        cx + Math.cos(angle) * radius,
        cy + Math.sin(angle) * radius,
        thickness / 2,
        color,
      );
    }
  }

  star(cx, cy, radius, color) {
    for (let y = -radius; y <= radius; y += 1) {
      for (let x = -radius; x <= radius; x += 1) {
        const ax = Math.abs(x);
        const ay = Math.abs(y);
        if (ax + ay <= radius || (ax <= 1 && ay <= radius) || (ay <= 1 && ax <= radius)) {
          this.set(cx + x, cy + y, color);
        }
      }
    }
  }
}

function save(name, canvas) {
  mkdirSync(OUT, { recursive: true });
  writeFileSync(join(OUT, name), encodePng(canvas));
  console.log(`gerado assets/sprites/${name} (${canvas.w}x${canvas.h})`);
}

// ---------------------------------------------------------------------------
// Enzo: 14 frames de 32x32
// ---------------------------------------------------------------------------

function part(canvas, ox, oy, rect, fill, outline = P.outline, inset = 1) {
  canvas.rect(ox + rect.x, oy + rect.y, rect.w, rect.h, outline);
  if (rect.w > inset * 2 && rect.h > inset * 2) {
    canvas.rect(
      ox + rect.x + inset,
      oy + rect.y + inset,
      rect.w - inset * 2,
      rect.h - inset * 2,
      fill,
    );
  }
}

function shiftRect(rect, dx, dy) {
  return { x: rect.x + dx, y: rect.y + dy, w: rect.w, h: rect.h };
}

function drawArm(canvas, ox, oy, arm, direction = "down") {
  part(canvas, ox, oy, arm, P.hoodie);
  let hand;
  if (direction === "right") {
    hand = {
      x: arm.x + arm.w - 1,
      y: arm.y + Math.floor((arm.h - 3) / 2),
      w: 3,
      h: 3,
    };
  } else if (direction === "left") {
    hand = {
      x: arm.x - 2,
      y: arm.y + Math.floor((arm.h - 3) / 2),
      w: 3,
      h: 3,
    };
  } else if (direction === "up") {
    hand = {
      x: arm.x + Math.floor((arm.w - 3) / 2),
      y: arm.y - 2,
      w: 3,
      h: 3,
    };
  } else {
    hand = {
      x: arm.x + Math.floor((arm.w - 3) / 2),
      y: arm.y + arm.h - 1,
      w: 3,
      h: 3,
    };
  }
  part(canvas, ox, oy, hand, P.skin);
}

function drawLeg(canvas, ox, oy, leg, shoeDx = 0) {
  part(canvas, ox, oy, leg, P.pants);
  const shoe = {
    x: leg.x - 1 + shoeDx,
    y: leg.y + leg.h - 1,
    w: leg.w + 2,
    h: 3,
  };
  part(canvas, ox, oy, shoe, P.shoe);
  canvas.rect(ox + shoe.x + 1, oy + shoe.y + 1, shoe.w - 2, 1, P.shoeLight);
}

function drawHair(canvas, ox, oy, head) {
  const x = head.x;
  const y = head.y;

  canvas.rect(ox + x - 1, oy + y - 1, head.w + 2, 3, P.hair);
  canvas.rect(ox + x, oy + y + 2, 2, 4, P.hair);
  canvas.rect(ox + x + head.w - 2, oy + y + 2, 2, 4, P.hair);

  const spikes = [
    [-2, 1],
    [0, -1],
    [2, -3],
    [5, -4],
    [8, -3],
    [11, -3],
    [14, -1],
    [15, 1],
    [3, 0],
    [7, -1],
    [10, 0],
  ];
  for (const [sx, sy] of spikes) {
    canvas.set(ox + x + sx, oy + y + sy, P.hair);
  }

  canvas.rect(ox + x + 2, oy + y, 5, 1, P.hairLight);
  canvas.set(ox + x + 9, oy + y - 1, P.hairLight);
  canvas.rect(ox + x - 1, oy + y + 3, 1, 3, P.hair);
  canvas.rect(ox + x + head.w, oy + y + 3, 1, 3, P.hair);
}

function drawFace(canvas, ox, oy, head, eyes, mouth) {
  const x = head.x;
  const y = head.y;
  const eyeY = y + 5;

  if (eyes === "hurt") {
    for (const ex of [x + 3, x + 9]) {
      canvas.set(ox + ex, oy + eyeY, P.eye);
      canvas.set(ox + ex + 1, oy + eyeY + 1, P.eye);
      canvas.set(ox + ex + 1, oy + eyeY, P.eye);
      canvas.set(ox + ex, oy + eyeY + 1, P.eye);
    }
  } else if (eyes === "closed") {
    canvas.rect(ox + x + 3, oy + eyeY + 1, 2, 1, P.eye);
    canvas.rect(ox + x + 9, oy + eyeY + 1, 2, 1, P.eye);
  } else if (eyes === "focus") {
    canvas.rect(ox + x + 3, oy + eyeY, 2, 1, P.eye);
    canvas.rect(ox + x + 9, oy + eyeY, 2, 1, P.eye);
    canvas.set(ox + x + 2, oy + eyeY - 1, P.hair);
    canvas.set(ox + x + 10, oy + eyeY - 1, P.hair);
  } else {
    canvas.rect(ox + x + 3, oy + eyeY, 2, 2, P.eye);
    canvas.rect(ox + x + 9, oy + eyeY, 2, 2, P.eye);
    canvas.set(ox + x + 3, oy + eyeY, P.white);
    canvas.set(ox + x + 9, oy + eyeY, P.white);
  }

  canvas.set(ox + x + 6, oy + y + 7, P.skinShade);

  if (mouth === "open") {
    canvas.rect(ox + x + 5, oy + y + 9, 4, 2, P.eye);
  } else if (mouth === "grin") {
    canvas.rect(ox + x + 5, oy + y + 9, 4, 1, P.eye);
    canvas.set(ox + x + 4, oy + y + 8, P.eye);
    canvas.set(ox + x + 9, oy + y + 8, P.eye);
  } else {
    canvas.rect(ox + x + 5, oy + y + 9, 3, 1, P.eye);
  }

  canvas.set(ox + x + 2, oy + y + 8, P.skinShade);
  canvas.set(ox + x + 11, oy + y + 8, P.skinShade);
}

function drawEnzo(canvas, ox, oy, options = {}) {
  const bob = options.bob ?? 0;
  const lean = options.lean ?? 0;
  const legL = options.legL ?? { x: 10, y: 24, w: 5, h: 6 };
  const legR = options.legR ?? { x: 17, y: 24, w: 5, h: 6 };
  const armL = options.armL ?? { x: 5, y: 16, w: 4, h: 8 };
  const armR = options.armR ?? { x: 23, y: 16, w: 4, h: 8 };
  const torso = options.torso ?? { x: 8, y: 15, w: 16, h: 9 };
  const head = options.head ?? { x: 9, y: 4, w: 14, h: 12 };

  // Braço e perna de trás.
  drawArm(
    canvas,
    ox,
    oy,
    shiftRect(armL, lean, bob),
    options.handLDir ?? "down",
  );
  drawLeg(canvas, ox, oy, legL, options.shoeLDx ?? 0);

  // Tronco.
  const torsoRect = shiftRect(torso, lean, bob);
  part(canvas, ox, oy, torsoRect, P.hoodie);
  canvas.rect(ox + torsoRect.x + 3, oy + torsoRect.y - 1, 10, 2, P.hoodieDark);
  canvas.rect(ox + torsoRect.x + 7, oy + torsoRect.y + 2, 2, torsoRect.h - 3, P.hoodieDark);
  canvas.rect(ox + torsoRect.x + 4, oy + torsoRect.y + torsoRect.h - 4, 8, 1, P.hoodieLight);
  canvas.rect(ox + torsoRect.x + 4, oy + torsoRect.y + torsoRect.h - 3, 8, 1, P.hoodieDark);

  // Perna da frente.
  drawLeg(canvas, ox, oy, legR, options.shoeRDx ?? 0);

  // Cabeça.
  const headRect = shiftRect(head, lean, bob);
  part(canvas, ox, oy, headRect, P.skin);
  canvas.rect(ox + headRect.x - 1, oy + headRect.y + 5, 2, 3, P.skinShade);
  canvas.rect(ox + headRect.x + headRect.w - 1, oy + headRect.y + 5, 2, 3, P.skinShade);

  drawHair(canvas, ox, oy, headRect);
  drawFace(
    canvas,
    ox,
    oy,
    headRect,
    options.eyes ?? "normal",
    options.mouth ?? "normal",
  );

  // Braço da frente.
  drawArm(
    canvas,
    ox,
    oy,
    shiftRect(armR, lean, bob),
    options.handRDir ?? "down",
  );

  if (options.aura) {
    const phase = options.auraPhase ?? 0;
    const points = [
      [3, 17],
      [26, 11],
      [2, 25],
      [28, 24],
      [6, 5],
      [25, 3],
      [14, 2],
      [29, 16],
    ];
    points.forEach(([px, py], index) => {
      if ((index + phase) % 2 === 0) {
        canvas.set(ox + px, oy + py, P.aura);
        canvas.set(ox + px + 1, oy + py, P.auraLight);
      }
    });
    canvas.circle(ox + 16, oy + 20, 2, P.aura);
    canvas.circle(ox + 16, oy + 20, 1, P.auraLight);
  }
}

function buildEnzoSheet() {
  const frames = 14;
  const canvas = new Canvas(32 * frames, 32);
  const poses = [
    // 0-1: idle
    { bob: 0 },
    { bob: 1, eyes: "normal" },
    // 2-5: corrida
    {
      bob: 0,
      legL: { x: 9, y: 24, w: 5, h: 6 },
      legR: { x: 18, y: 24, w: 5, h: 6 },
      armL: { x: 4, y: 17, w: 4, h: 7 },
      armR: { x: 24, y: 15, w: 4, h: 7 },
      handLDir: "down",
      handRDir: "down",
    },
    {
      bob: 1,
      legL: { x: 11, y: 23, w: 5, h: 5 },
      legR: { x: 16, y: 25, w: 5, h: 5 },
      armL: { x: 6, y: 15, w: 4, h: 7 },
      armR: { x: 23, y: 17, w: 4, h: 7 },
    },
    {
      bob: 0,
      legL: { x: 8, y: 24, w: 5, h: 6 },
      legR: { x: 19, y: 24, w: 5, h: 6 },
      armL: { x: 4, y: 15, w: 4, h: 7 },
      armR: { x: 24, y: 17, w: 4, h: 7 },
    },
    {
      bob: 1,
      legL: { x: 12, y: 25, w: 5, h: 5 },
      legR: { x: 15, y: 23, w: 5, h: 5 },
      armL: { x: 6, y: 17, w: 4, h: 7 },
      armR: { x: 23, y: 15, w: 4, h: 7 },
    },
    // 6: pulo
    {
      bob: 0,
      legL: { x: 9, y: 22, w: 5, h: 4 },
      legR: { x: 18, y: 22, w: 5, h: 4 },
      armL: { x: 5, y: 14, w: 4, h: 6 },
      armR: { x: 23, y: 14, w: 4, h: 6 },
      handLDir: "up",
      handRDir: "up",
      eyes: "focus",
    },
    // 7: queda
    {
      bob: 0,
      legL: { x: 7, y: 24, w: 5, h: 6 },
      legR: { x: 20, y: 24, w: 5, h: 6 },
      armL: { x: 3, y: 15, w: 5, h: 4 },
      armR: { x: 24, y: 15, w: 5, h: 4 },
      handLDir: "left",
      handRDir: "right",
      mouth: "open",
    },
    // 8: dash
    {
      lean: 2,
      legL: { x: 8, y: 25, w: 5, h: 5 },
      legR: { x: 14, y: 24, w: 5, h: 6 },
      armL: { x: 5, y: 16, w: 4, h: 7 },
      armR: { x: 24, y: 18, w: 6, h: 4 },
      handRDir: "right",
      eyes: "focus",
      mouth: "grin",
    },
    // 9: dano
    {
      lean: -1,
      legL: { x: 8, y: 24, w: 5, h: 6 },
      legR: { x: 19, y: 24, w: 5, h: 6 },
      armL: { x: 3, y: 14, w: 5, h: 4 },
      armR: { x: 24, y: 14, w: 5, h: 4 },
      handLDir: "left",
      handRDir: "right",
      eyes: "hurt",
      mouth: "open",
    },
    // 10-11: ataque
    {
      lean: 1,
      legL: { x: 9, y: 24, w: 5, h: 6 },
      legR: { x: 18, y: 24, w: 5, h: 6 },
      armL: { x: 4, y: 16, w: 4, h: 7 },
      armR: { x: 23, y: 16, w: 7, h: 5 },
      handRDir: "right",
      eyes: "focus",
      mouth: "grin",
    },
    {
      lean: 1,
      legL: { x: 8, y: 24, w: 5, h: 6 },
      legR: { x: 19, y: 24, w: 5, h: 6 },
      armL: { x: 4, y: 15, w: 4, h: 7 },
      armR: { x: 23, y: 18, w: 8, h: 5 },
      handRDir: "right",
      eyes: "focus",
      mouth: "open",
    },
    // 12-13: farmando aura
    {
      bob: 0,
      legL: { x: 10, y: 24, w: 5, h: 6 },
      legR: { x: 17, y: 24, w: 5, h: 6 },
      armL: { x: 6, y: 18, w: 4, h: 5 },
      armR: { x: 22, y: 18, w: 4, h: 5 },
      handLDir: "right",
      handRDir: "left",
      eyes: "closed",
      aura: true,
      auraPhase: 0,
    },
    {
      bob: 1,
      legL: { x: 10, y: 24, w: 5, h: 6 },
      legR: { x: 17, y: 24, w: 5, h: 6 },
      armL: { x: 6, y: 18, w: 4, h: 5 },
      armR: { x: 22, y: 18, w: 4, h: 5 },
      handLDir: "right",
      handRDir: "left",
      eyes: "closed",
      aura: true,
      auraPhase: 1,
    },
  ];

  poses.forEach((pose, index) => {
    drawEnzo(canvas, index * 32, 0, pose);
  });
  return canvas;
}

// ---------------------------------------------------------------------------
// Capanga: 4 frames de 32x32
// ---------------------------------------------------------------------------

function drawSlime(canvas, ox, oy, options = {}) {
  const cx = 16;
  const cy = options.cy ?? 21;
  const rx = options.rx ?? 11;
  const ry = options.ry ?? 9;
  const hurt = options.hurt ?? false;

  canvas.ellipse(ox + cx, oy + cy, rx + 1, ry + 1, P.outline);
  canvas.ellipse(ox + cx, oy + cy, rx, ry, hurt ? P.slimeHurt : P.slime);
  canvas.ellipse(
    ox + cx,
    oy + cy + Math.floor(ry * 0.45),
    Math.max(2, rx - 2),
    Math.max(2, Math.floor(ry * 0.55)),
    hurt ? "#86b97a" : P.slimeDark,
  );
  canvas.ellipse(
    ox + cx - 3,
    oy + cy - 3,
    Math.max(2, Math.floor(rx * 0.55)),
    Math.max(2, Math.floor(ry * 0.45)),
    hurt ? "#c8e8b8" : P.slimeLight,
  );
  canvas.circle(ox + cx - 5, oy + cy - 4, 2, hurt ? "#e8f8dc" : "#d8ffc4");

  if (hurt) {
    for (const ex of [cx - 5, cx + 4]) {
      canvas.set(ox + ex, oy + cy - 2, P.outline);
      canvas.set(ox + ex + 1, oy + cy - 1, P.outline);
      canvas.set(ox + ex + 1, oy + cy - 2, P.outline);
      canvas.set(ox + ex, oy + cy - 1, P.outline);
    }
  } else {
    for (const ex of [cx - 5, cx + 4]) {
      canvas.circle(ox + ex, oy + cy - 2, 2, P.white);
      canvas.set(ox + ex, oy + cy - 2, P.outline);
    }
  }

  canvas.rect(ox + cx - 2, oy + cy + 4, 4, 2, P.outline);
  canvas.set(ox + cx - 3, oy + cy + 3, P.outline);
  canvas.set(ox + cx + 2, oy + cy + 3, P.outline);

  // Gotas e brilho.
  canvas.set(ox + cx - rx + 1, oy + cy + ry - 1, P.slimeDark);
  canvas.set(ox + cx + rx - 1, oy + cy + ry - 2, P.slimeDark);
  canvas.set(ox + cx + rx - 3, oy + cy + ry + 1, P.slimeDark);
  canvas.set(ox + cx - 4, oy + cy - ry + 1, P.white);
}

function buildSlimeSheet() {
  const canvas = new Canvas(32 * 4, 32);
  drawSlime(canvas, 0, 0, { cy: 21, rx: 11, ry: 9 });
  drawSlime(canvas, 32, 0, { cy: 23, rx: 12, ry: 7 });
  drawSlime(canvas, 64, 0, { cy: 20, rx: 10, ry: 11 });
  drawSlime(canvas, 96, 0, { cy: 21, rx: 11, ry: 9, hurt: true });
  return canvas;
}

// ---------------------------------------------------------------------------
// Tiles: 4 frames de 16x16
// ---------------------------------------------------------------------------

function drawGroundTop(canvas, ox) {
  canvas.rect(ox, 0, 16, 16, P.dirt);
  canvas.rect(ox, 3, 16, 13, P.dirt);

  canvas.rect(ox, 0, 16, 2, P.grassLight);
  canvas.rect(ox, 2, 16, 2, P.grass);
  canvas.rect(ox, 4, 16, 1, P.grassDark);

  const blades = [1, 4, 7, 10, 13];
  blades.forEach((x, index) => {
    canvas.set(ox + x, 0, P.grassLight);
    if (index % 2 === 0) canvas.set(ox + x, -1, P.grassLight);
  });

  canvas.rect(ox + 2, 7, 2, 1, P.dirtLight);
  canvas.rect(ox + 9, 10, 3, 1, P.dirtLight);
  canvas.rect(ox + 12, 6, 1, 2, P.dirtDark);
  canvas.rect(ox + 4, 12, 2, 2, P.dirtDark);
  canvas.set(ox + 7, 9, P.dirtLight);
}

function drawGroundFill(canvas, ox) {
  canvas.rect(ox, 0, 16, 16, P.dirt);
  canvas.rect(ox, 0, 16, 1, P.dirtDark);
  canvas.rect(ox + 2, 3, 3, 1, P.dirtLight);
  canvas.rect(ox + 9, 5, 2, 2, P.dirtDark);
  canvas.rect(ox + 5, 9, 3, 1, P.dirtLight);
  canvas.rect(ox + 12, 12, 2, 1, P.dirtDark);
  canvas.set(ox + 1, 13, P.dirtLight);
  canvas.set(ox + 8, 14, P.dirtDark);
}

function drawBlock(canvas, ox) {
  canvas.rect(ox, 0, 16, 16, P.stoneDark);
  canvas.rect(ox + 1, 1, 14, 14, P.stone);
  canvas.rect(ox + 1, 1, 14, 2, P.stoneLight);
  canvas.rect(ox + 1, 1, 2, 14, P.stoneLight);
  canvas.rect(ox + 1, 13, 14, 2, P.stoneDark);
  canvas.rect(ox + 13, 1, 2, 14, P.stoneDark);
  canvas.rect(ox + 4, 5, 2, 1, P.stoneDark);
  canvas.rect(ox + 9, 4, 1, 3, P.stoneDark);
  canvas.rect(ox + 7, 9, 4, 1, P.stoneDark);
  canvas.rect(ox + 5, 10, 1, 2, P.stoneDark);
  canvas.rect(ox + 11, 11, 2, 1, P.stoneLight);
}

function drawPlatform(canvas, ox) {
  canvas.rect(ox, 0, 16, 16, P.woodDark);
  canvas.rect(ox + 1, 1, 14, 5, P.wood);
  canvas.rect(ox + 1, 7, 14, 7, P.wood);
  canvas.rect(ox + 1, 1, 14, 1, P.woodLight);
  canvas.rect(ox + 1, 7, 14, 1, P.woodLight);
  canvas.rect(ox + 4, 2, 1, 5, P.woodDark);
  canvas.rect(ox + 11, 2, 1, 5, P.woodDark);
  canvas.rect(ox + 7, 8, 1, 6, P.woodDark);
  canvas.set(ox + 2, 3, P.woodDark);
  canvas.set(ox + 13, 9, P.woodDark);
  canvas.set(ox + 13, 4, P.woodLight);
  canvas.set(ox + 3, 11, P.woodLight);
}

function buildTilesSheet() {
  const canvas = new Canvas(16 * 4, 16);
  drawGroundTop(canvas, 0);
  drawGroundFill(canvas, 16);
  drawBlock(canvas, 32);
  drawPlatform(canvas, 48);
  return canvas;
}

// ---------------------------------------------------------------------------
// FX: 8 frames de 32x32 (Mogar, slash, faíscas, poeira)
// ---------------------------------------------------------------------------

function drawMogar(canvas, ox, oy) {
  const cx = 16;
  const cy = 16;

  canvas.ellipse(ox + cx, oy + cy, 14, 9, "#6b3cff44");
  canvas.ellipse(ox + cx, oy + cy, 12, 7, "#8a5cff88");
  canvas.ellipse(ox + cx, oy + cy, 9, 5, "#b48cffcc");
  canvas.ellipse(ox + cx, oy + cy, 6, 3, "#e8dcff");
  canvas.ellipse(ox + cx, oy + cy, 3, 2, "#ffffff");

  // Cauda de energia.
  for (let i = 1; i <= 9; i += 1) {
    const alpha = Math.round(180 - i * 18);
    canvas.circle(ox + cx - 8 - i * 1.4, oy + cy + Math.sin(i * 0.8) * 1.5, 2.4, `#5ef2ff${alpha.toString(16).padStart(2, "0")}`);
  }

  // Faíscas.
  const sparks = [
    [5, 9],
    [27, 10],
    [7, 23],
    [25, 22],
    [16, 4],
    [16, 28],
  ];
  sparks.forEach(([x, y], index) => {
    canvas.set(ox + x, oy + y, index % 2 === 0 ? P.aura : P.auraLight);
  });
}

function drawSlash(canvas, ox, oy, radius, thickness, color, glow) {
  const cx = 16;
  const cy = 16;
  canvas.arc(ox + cx, oy + cy, radius, -1.15, 1.15, thickness + 3, glow);
  canvas.arc(ox + cx, oy + cy, radius, -1.05, 1.05, thickness, color);
  canvas.arc(ox + cx, oy + cy, radius - 3, -0.8, 0.8, 1, "#ffffff");

  const sparks = [
    [cx + radius - 1, cy - radius + 2],
    [cx + radius + 2, cy],
    [cx + radius - 1, cy + radius - 2],
  ];
  sparks.forEach(([x, y], index) => {
    canvas.set(ox + x, oy + y, index === 1 ? "#ffffff" : color);
  });
}

function drawSpark(canvas, ox, oy, color) {
  const cx = 16;
  const cy = 16;
  canvas.circle(ox + cx, oy + cy, 6, `${color}33`);
  canvas.star(ox + cx, oy + cy, 5, color);
  canvas.star(ox + cx, oy + cy, 2, "#ffffff");
}

function drawDust(canvas, ox, oy) {
  const cx = 16;
  const cy = 20;
  canvas.ellipse(ox + cx - 5, oy + cy, 5, 3, "#d8d0c044");
  canvas.ellipse(ox + cx + 4, oy + cy - 1, 6, 4, "#c8c0b055");
  canvas.ellipse(ox + cx, oy + cy + 2, 7, 3, "#b8b0a044");
  canvas.set(ox + cx - 8, oy + cy - 2, "#e8e0d066");
  canvas.set(ox + cx + 9, oy + cy + 1, "#e8e0d066");
}

function buildFxSheet() {
  const canvas = new Canvas(32 * 8, 32);
  drawMogar(canvas, 0, 0);
  drawSlash(canvas, 32, 0, 9, 3, "#bffcff", "#5ef2ff55");
  drawSlash(canvas, 64, 0, 11, 4, "#7ef0ff", "#5ef2ff66");
  drawSlash(canvas, 96, 0, 13, 4, "#5ef2ff", "#3a7bd588");
  drawSpark(canvas, 128, 0, "#f6c445");
  drawSpark(canvas, 160, 0, "#5ef2ff");
  drawSpark(canvas, 192, 0, "#ff6b5e");
  drawDust(canvas, 224, 0);
  return canvas;
}

// ---------------------------------------------------------------------------
// Bandeira: 32x64
// ---------------------------------------------------------------------------

function buildFlag() {
  const canvas = new Canvas(32, 64);

  // Mastro.
  canvas.rect(5, 3, 4, 61, P.outline);
  canvas.rect(6, 4, 2, 59, "#8f93a8");
  canvas.rect(6, 4, 1, 59, "#c7cbe0");

  // Base.
  canvas.rect(2, 60, 10, 4, P.outline);
  canvas.rect(3, 61, 8, 2, P.goldDark);

  // Esfera no topo.
  canvas.circle(6, 3, 3, P.outline);
  canvas.circle(6, 3, 2, P.gold);
  canvas.set(5, 2, P.goldLight);

  // Bandeira ondulada.
  for (let y = 5; y <= 29; y += 1) {
    const wave = Math.round(Math.sin((y - 5) * 0.38) * 1.5);
    const right = 30 - Math.abs(wave);
    canvas.rect(9, y, right - 9, 1, P.goldDark);
    canvas.rect(10, y, right - 11, 1, y < 16 ? P.gold : P.goldDark);
  }
  canvas.rect(9, 5, 22, 1, P.goldLight);
  canvas.rect(9, 29, 22, 1, P.goldDark);

  // Emblema: estrela de quatro pontas.
  canvas.star(19, 17, 6, P.goldDark);
  canvas.star(19, 17, 5, P.goldLight);
  canvas.star(19, 17, 2, P.gold);
  canvas.set(19, 17, P.white);

  // Brilho.
  canvas.set(11, 7, P.white);
  canvas.set(12, 7, P.goldLight);
  canvas.set(28, 25, P.goldLight);

  return canvas;
}

// ---------------------------------------------------------------------------
// Itens: 5 frames de 16x16
// ---------------------------------------------------------------------------

function buildItemsSheet() {
  const canvas = new Canvas(16 * 5, 16);

  // Espada.
  let ox = 0;
  canvas.line(ox + 3, 12, ox + 11, 4, P.outline, 3);
  canvas.line(ox + 3, 12, ox + 11, 4, P.steel, 1.5);
  canvas.line(ox + 4, 12, ox + 11, 5, P.white, 1);
  canvas.line(ox + 8, 7, ox + 12, 3, P.steelDark, 1);
  canvas.rect(ox + 2, 12, 3, 2, P.gold);
  canvas.rect(ox + 3, 13, 2, 2, P.woodDark);
  canvas.set(ox + 4, 14, P.woodLight);

  // Arco.
  ox = 16;
  canvas.arc(ox + 6, 8, 5, -1.2, 1.2, 1.5, P.wood);
  canvas.arc(ox + 6, 8, 4, -1.1, 1.1, 1, P.woodLight);
  canvas.line(ox + 8, 3, ox + 8, 13, P.white, 1);
  canvas.line(ox + 8, 8, ox + 12, 8, P.woodDark, 1);
  canvas.set(ox + 12, 8, P.steel);

  // Armadura.
  ox = 32;
  canvas.rect(ox + 3, 3, 10, 10, P.outline);
  canvas.rect(ox + 4, 4, 8, 8, P.pants);
  canvas.rect(ox + 4, 4, 8, 1, P.steel);
  canvas.rect(ox + 5, 5, 2, 6, P.pantsDark);
  canvas.rect(ox + 9, 5, 2, 6, P.pantsDark);
  canvas.rect(ox + 7, 5, 2, 7, P.steelDark);
  canvas.rect(ox + 5, 11, 6, 1, P.gold);
  canvas.set(ox + 7, 7, P.goldLight);

  // Amuleto.
  ox = 48;
  canvas.arc(ox + 8, 5, 4, 3.4, 6.1, 1, P.goldDark);
  canvas.arc(ox + 8, 5, 3.5, 3.5, 6.0, 1, P.gold);
  canvas.line(ox + 8, 8, ox + 8, 10, P.goldDark, 1);
  canvas.ellipse(ox + 8, 12, 3, 3, P.outline);
  canvas.ellipse(ox + 8, 12, 2, 2, P.gem);
  canvas.set(ox + 7, 11, P.gemLight);

  // Poção.
  ox = 64;
  canvas.rect(ox + 6, 2, 4, 2, P.outline);
  canvas.rect(ox + 7, 1, 2, 2, P.woodLight);
  canvas.rect(ox + 5, 4, 6, 2, P.outline);
  canvas.rect(ox + 6, 4, 4, 2, P.white);
  canvas.ellipse(ox + 8, 10, 5, 5, P.outline);
  canvas.ellipse(ox + 8, 10, 4, 4, P.potion);
  canvas.ellipse(ox + 8, 9, 3, 2, P.potionLight);
  canvas.set(ox + 6, 9, P.white);
  canvas.set(ox + 5, 6, P.steelDark);

  return canvas;
}

// ---------------------------------------------------------------------------
// Execução
// ---------------------------------------------------------------------------

mkdirSync(OUT, { recursive: true });
save("tiles.png", buildTilesSheet());
save("enzo.png", buildEnzoSheet());
save("slime.png", buildSlimeSheet());
save("fx.png", buildFxSheet());
save("flag.png", buildFlag());
save("items.png", buildItemsSheet());
console.log("sprites ok");

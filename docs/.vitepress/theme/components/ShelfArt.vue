<script setup lang="ts">
// A small pixel bookcase. Every unit is one "pixel"; the SVG scales crisply.
type Rect = { x: number; y: number; w: number; h: number; c: string }

const C = {
  ochre: '#c98a2e', ochreD: '#94601a',
  sage: '#5f8a6c', sageD: '#3f6049',
  slate: '#4d6b7e', slateD: '#334a58',
  brick: '#b0502f', brickD: '#7a3119',
  plum: '#7c5467', plumD: '#573a48',
  cream: '#e8dcbc', creamD: '#c4b48e',
  kraft: '#b99467', kraftD: '#8c6c45',
}

const rects: Rect[] = []
const r = (x: number, y: number, w: number, h: number, c: string) => rects.push({ x, y, w, h, c })

// A book standing on `floor`, with a couple of bands on its spine.
function book(x: number, floor: number, w: number, h: number, c: string, d: string, bands = [2, 4]) {
  const top = floor - h
  r(x, top, w, h, c)
  r(x + w - 1, top, 1, h, d) // shaded edge
  for (const b of bands) r(x, top + b, w - 1, 1, d)
}

const S1 = 21 // first shelf floor
const S2 = 43 // second shelf floor

// --- Upper shelf: books, a stack, a floppy disk
book(6, S1, 3, 13, C.ochre, C.ochreD)
book(9, S1, 4, 15, C.sage, C.sageD, [2, 11])
book(13, S1, 2, 11, C.cream, C.creamD, [2])
book(15, S1, 3, 14, C.brick, C.brickD, [3, 5])
book(18, S1, 3, 12, C.slate, C.slateD, [2])

// lying stack
r(24, S1 - 2, 15, 2, C.plum); r(24, S1 - 2, 15, 1, C.plumD)
r(25, S1 - 4, 13, 2, C.ochre); r(25, S1 - 4, 13, 1, C.ochreD)
r(24, S1 - 6, 14, 2, C.sage); r(24, S1 - 6, 14, 1, C.sageD)

// floppy disk on the stack
const fx = 27, fy = S1 - 15
r(fx, fy, 9, 9, C.slate)
r(fx + 2, fy, 5, 3, C.cream); r(fx + 5, fy + 1, 1, 1, C.slateD)
r(fx + 1, fy + 5, 7, 4, C.cream); r(fx + 2, fy + 6, 5, 1, C.creamD)
r(fx + 8, fy, 1, 9, C.slateD)

book(43, S1, 3, 14, C.plum, C.plumD, [2, 10])
book(46, S1, 3, 12, C.ochre, C.ochreD)
book(49, S1, 4, 15, C.slate, C.slateD, [3, 12])
book(53, S1, 2, 10, C.sage, C.sageD, [2])
book(55, S1, 3, 13, C.cream, C.creamD, [2, 4])

// --- Lower shelf: archive boxes, books, a pile of bits
function box(x: number, floor: number, w: number, h: number, c: string, d: string) {
  const top = floor - h
  r(x, top, w, h, c)
  r(x, top, w, 1, d)
  r(x + w - 1, top, 1, h, d)
  r(x + 2, top + 3, w - 5, 3, C.cream) // label holder
  r(x + 2, top + 5, w - 5, 1, C.creamD)
  r(x + Math.floor(w / 2) - 2, top + h - 3, 3, 1, d) // hand hole
}
box(6, S2, 11, 11, C.kraft, C.kraftD)
box(17, S2, 11, 11, C.creamD, C.kraftD)

book(30, S2, 3, 14, C.brick, C.brickD, [2, 11])
book(33, S2, 3, 13, C.sage, C.sageD)
book(36, S2, 2, 15, C.ochre, C.ochreD, [3])

// bits: small cubes, stacked
function bit(x: number, y: number, c: string, d: string) {
  r(x, y, 5, 5, c)
  r(x, y, 5, 1, d)
  r(x + 4, y, 1, 5, d)
}
bit(42, S2 - 5, C.cream, C.ochre)
bit(47, S2 - 5, C.cream, C.sage)
bit(52, S2 - 5, C.cream, C.brick)
bit(44, S2 - 10, C.cream, C.slate)
bit(49, S2 - 10, C.cream, C.ochre)
bit(47, S2 - 15, C.ochre, C.ochreD)
</script>

<template>
  <svg
    class="shelf-art"
    viewBox="0 0 64 50"
    shape-rendering="crispEdges"
    role="img"
    aria-label="A pixel-art bookcase holding books, a floppy disk, archive boxes, and a stack of small cubes"
  >
    <!-- case -->
    <rect class="back" x="3" y="3" width="58" height="42" />
    <rect class="shadow" x="4" y="4" width="56" height="1" />
    <rect class="shadow" x="4" y="24" width="56" height="1" />
    <g>
      <rect
        v-for="(b, i) in rects"
        :key="i"
        :x="b.x"
        :y="b.y"
        :width="b.w"
        :height="b.h"
        :fill="b.c"
      />
    </g>
    <rect class="wood" x="1" y="1" width="62" height="2" />
    <rect class="wood" x="1" y="1" width="2" height="47" />
    <rect class="wood" x="61" y="1" width="2" height="47" />
    <rect class="wood" x="3" y="21" width="58" height="3" />
    <rect class="wood-d" x="3" y="23" width="58" height="1" />
    <rect class="wood" x="3" y="43" width="58" height="3" />
    <rect class="wood-d" x="3" y="45" width="58" height="1" />
    <rect class="wood-d" x="1" y="46" width="62" height="2" />
    <!-- shelf labels -->
    <rect class="label" x="26" y="21.5" width="12" height="2" />
    <text class="label-text" x="32" y="23.05">notes/</text>
    <rect class="label" x="26" y="43.5" width="12" height="2" />
    <text class="label-text" x="32" y="45.05">archive/</text>
  </svg>
</template>

<style scoped>
.shelf-art {
  display: block;
  width: 100%;
  height: auto;
  image-rendering: pixelated;
}

.back {
  fill: var(--bs-paper-3);
}

.shadow {
  fill: rgba(0, 0, 0, 0.08);
}

.wood {
  fill: #8a6644;
}

.wood-d {
  fill: #5c432b;
}

.dark .back {
  fill: #1e1b15;
}

.dark .shadow {
  fill: rgba(0, 0, 0, 0.35);
}

.label {
  fill: #efe5c9;
}

.label-text {
  font-family: var(--vp-font-family-mono);
  font-size: 1.45px;
  font-weight: 600;
  fill: #3a3027;
  text-anchor: middle;
}
</style>

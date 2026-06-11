/**
 * Color utility module for Sailfish Studio editor.
 * Provides hex/RGB conversion, category color mapping, and color blending.
 */

/** Category color map matching Scratch 3.0 block category colors */
const CATEGORY_COLORS: Record<string, string> = {
  motion: '#4C97FF',
  looks: '#9966FF',
  sound: '#CF63CF',
  events: '#FFBF00',
  control: '#FFAB19',
  sensing: '#5CB1D6',
  operators: '#59C059',
  variables: '#FF8C1A',
  pen: '#0fBD8C',
};

/**
 * Convert a hex color string to RGB components.
 * Supports 3-digit (#RGB) and 6-digit (#RRGGBB) formats.
 */
export function hexToRgb(hex: string): { r: number; g: number; b: number } {
  const cleaned = hex.replace(/^#/, '');

  if (cleaned.length === 3) {
    const r = parseInt(cleaned[0] + cleaned[0], 16);
    const g = parseInt(cleaned[1] + cleaned[1], 16);
    const b = parseInt(cleaned[2] + cleaned[2], 16);
    if (isNaN(r) || isNaN(g) || isNaN(b)) {
      throw new Error(`Invalid hex color: ${hex}`);
    }
    return { r, g, b };
  }

  if (cleaned.length === 6) {
    const r = parseInt(cleaned.substring(0, 2), 16);
    const g = parseInt(cleaned.substring(2, 4), 16);
    const b = parseInt(cleaned.substring(4, 6), 16);
    if (isNaN(r) || isNaN(g) || isNaN(b)) {
      throw new Error(`Invalid hex color: ${hex}`);
    }
    return { r, g, b };
  }

  throw new Error(`Invalid hex color: ${hex}`);
}

/**
 * Convert RGB components to a hex color string.
 */
export function rgbToHex(r: number, g: number, b: number): string {
  if (r < 0 || r > 255 || g < 0 || g > 255 || b < 0 || b > 255) {
    throw new Error(`RGB values must be between 0 and 255, got: r=${r}, g=${g}, b=${b}`);
  }
  const toHex = (n: number) => n.toString(16).padStart(2, '0').toUpperCase();
  return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}

/**
 * Return the hex color associated with a block category.
 * Falls back to a default gray for unknown categories.
 */
export function categoryColor(category: string): string {
  return CATEGORY_COLORS[category.toLowerCase()] ?? '#AAAAAA';
}

/**
 * Blend two hex colors together by a given ratio.
 * ratio=0 returns color1, ratio=1 returns color2.
 */
export function blendColors(color1: string, color2: string, ratio: number): string {
  if (ratio < 0 || ratio > 1) {
    throw new Error(`Ratio must be between 0 and 1, got: ${ratio}`);
  }
  const c1 = hexToRgb(color1);
  const c2 = hexToRgb(color2);
  const r = Math.round(c1.r + (c2.r - c1.r) * ratio);
  const g = Math.round(c1.g + (c2.g - c1.g) * ratio);
  const b = Math.round(c1.b + (c2.b - c1.b) * ratio);
  return rgbToHex(r, g, b);
}

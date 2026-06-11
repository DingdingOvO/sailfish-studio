import { describe, it, expect } from 'vitest';
import { hexToRgb, rgbToHex, categoryColor, blendColors } from './color';

describe('hexToRgb', () => {
  it('should convert 6-digit hex to RGB', () => {
    expect(hexToRgb('#4C97FF')).toEqual({ r: 76, g: 151, b: 255 });
  });

  it('should convert 3-digit hex to RGB', () => {
    expect(hexToRgb('#FFF')).toEqual({ r: 255, g: 255, b: 255 });
  });

  it('should convert black correctly', () => {
    expect(hexToRgb('#000000')).toEqual({ r: 0, g: 0, b: 0 });
  });

  it('should throw on invalid hex string', () => {
    expect(() => hexToRgb('#ZZZZZZ')).toThrow('Invalid hex color');
  });

  it('should throw on invalid length', () => {
    expect(() => hexToRgb('#1234')).toThrow('Invalid hex color');
  });
});

describe('rgbToHex', () => {
  it('should convert RGB to hex', () => {
    expect(rgbToHex(76, 151, 255)).toBe('#4C97FF');
  });

  it('should convert zero values', () => {
    expect(rgbToHex(0, 0, 0)).toBe('#000000');
  });

  it('should convert max values', () => {
    expect(rgbToHex(255, 255, 255)).toBe('#FFFFFF');
  });

  it('should throw on out-of-range values', () => {
    expect(() => rgbToHex(-1, 0, 0)).toThrow('RGB values must be between 0 and 255');
    expect(() => rgbToHex(0, 256, 0)).toThrow('RGB values must be between 0 and 255');
  });
});

describe('categoryColor', () => {
  it('should return correct color for motion category', () => {
    expect(categoryColor('motion')).toBe('#4C97FF');
  });

  it('should return correct color for looks category', () => {
    expect(categoryColor('looks')).toBe('#9966FF');
  });

  it('should return correct color for sound category', () => {
    expect(categoryColor('sound')).toBe('#CF63CF');
  });

  it('should return correct color for events category', () => {
    expect(categoryColor('events')).toBe('#FFBF00');
  });

  it('should return correct color for control category', () => {
    expect(categoryColor('control')).toBe('#FFAB19');
  });

  it('should return correct color for sensing category', () => {
    expect(categoryColor('sensing')).toBe('#5CB1D6');
  });

  it('should return correct color for operators category', () => {
    expect(categoryColor('operators')).toBe('#59C059');
  });

  it('should return correct color for variables category', () => {
    expect(categoryColor('variables')).toBe('#FF8C1A');
  });

  it('should return correct color for pen category', () => {
    expect(categoryColor('pen')).toBe('#0fBD8C');
  });

  it('should return default gray for unknown category', () => {
    expect(categoryColor('unknown')).toBe('#AAAAAA');
  });

  it('should be case-insensitive', () => {
    expect(categoryColor('Motion')).toBe('#4C97FF');
    expect(categoryColor('LOOKS')).toBe('#9966FF');
  });
});

describe('blendColors', () => {
  it('should return color1 when ratio is 0', () => {
    expect(blendColors('#000000', '#FFFFFF', 0)).toBe('#000000');
  });

  it('should return color2 when ratio is 1', () => {
    expect(blendColors('#000000', '#FFFFFF', 1)).toBe('#FFFFFF');
  });

  it('should blend colors at 50% ratio', () => {
    const result = blendColors('#000000', '#FFFFFF', 0.5);
    expect(result).toBe('#808080');
  });

  it('should blend two Scratch category colors', () => {
    const result = blendColors('#4C97FF', '#9966FF', 0.5);
    // r=114.5→115(0x73), g=126.5→127(0x7F), b=255(0xFF)
    expect(result).toBe('#737FFF');
  });

  it('should throw on ratio < 0', () => {
    expect(() => blendColors('#000000', '#FFFFFF', -0.1)).toThrow('Ratio must be between 0 and 1');
  });

  it('should throw on ratio > 1', () => {
    expect(() => blendColors('#000000', '#FFFFFF', 1.5)).toThrow('Ratio must be between 0 and 1');
  });
});

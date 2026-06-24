/** A palette is a map of CSS-variable color channels expressed as "r g b". */
export interface Palette {
  bg: string;
  surface: string;
  surface2: string;
  border: string;
  text: string;
  textDim: string;
  primary: string;
  accent: string;
  danger: string;
  success: string;
  warning: string;
}

export interface PalettePreset {
  id: string;
  name: string;
  palette: Palette;
}

export const PRESETS: PalettePreset[] = [
  {
    id: "amber-crt",
    name: "Amber CRT",
    palette: {
      bg: "13 12 10",
      surface: "26 23 18",
      surface2: "38 33 24",
      border: "82 66 38",
      text: "245 214 150",
      textDim: "168 140 92",
      primary: "255 176 0",
      accent: "255 213 102",
      danger: "240 92 64",
      success: "120 200 80",
      warning: "255 176 0",
    },
  },
  {
    id: "green-phosphor",
    name: "Green Phosphor",
    palette: {
      bg: "8 12 9",
      surface: "14 22 16",
      surface2: "20 32 23",
      border: "38 74 46",
      text: "180 255 190",
      textDim: "104 168 116",
      primary: "57 255 120",
      accent: "150 255 180",
      danger: "255 90 90",
      success: "57 255 120",
      warning: "230 255 120",
    },
  },
  {
    id: "synthwave",
    name: "Synthwave 84",
    palette: {
      bg: "16 10 28",
      surface: "26 16 44",
      surface2: "40 24 64",
      border: "92 50 130",
      text: "240 220 255",
      textDim: "168 130 210",
      primary: "255 64 160",
      accent: "60 220 240",
      danger: "255 80 110",
      success: "120 240 180",
      warning: "255 200 80",
    },
  },
  {
    id: "ibm-noir",
    name: "IBM Noir",
    palette: {
      bg: "10 12 16",
      surface: "18 22 28",
      surface2: "28 34 42",
      border: "58 66 78",
      text: "210 220 230",
      textDim: "130 144 160",
      primary: "120 200 255",
      accent: "120 255 220",
      danger: "248 90 82",
      success: "90 220 140",
      warning: "245 200 90",
    },
  },
  {
    id: "commodore",
    name: "Commodore 64",
    palette: {
      bg: "20 16 64",
      surface: "32 28 92",
      surface2: "48 44 120",
      border: "108 100 200",
      text: "188 184 255",
      textDim: "134 130 210",
      primary: "150 220 255",
      accent: "255 255 160",
      danger: "255 110 110",
      success: "150 240 170",
      warning: "255 220 130",
    },
  },
];

export const DEFAULT_PRESET = PRESETS[0];

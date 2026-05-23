type TabVisual = {
  backgroundColor: string;
  textColor: string;
};

const tabPalette: TabVisual[] = [
  { backgroundColor: "#333333", textColor: "#ffffff" },
  { backgroundColor: "#8A4760", textColor: "#ffffff" },
  { backgroundColor: "#087F73", textColor: "#ffffff" },
  { backgroundColor: "#9BD8A0", textColor: "#ffffff" },
  { backgroundColor: "#a16207", textColor: "#ffffff" },
  { backgroundColor: "#7c3aed", textColor: "#ffffff" },
  { backgroundColor: "#0369a1", textColor: "#ffffff" },
  { backgroundColor: "#166534", textColor: "#ffffff" },
];

export function getTabVisual(tabId: number): TabVisual {
  const index = Math.abs(tabId - 1) % tabPalette.length;
  return tabPalette[index];
}
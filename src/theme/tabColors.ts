type TabVisual = {
  backgroundColor: string;
  textColor: string;
};

const tabPalette: TabVisual[] = [
  { backgroundColor: "#1E1E1E", textColor: "#ffffff" },
  { backgroundColor: "#8A4760", textColor: "#ffffff" },
  { backgroundColor: "#087F73", textColor: "#ffffff" },
  { backgroundColor: "#018F2F", textColor: "#ffffff" },
  { backgroundColor: "#FF9500", textColor: "#ffffff" },
  { backgroundColor: "#009F9A", textColor: "#ffffff" },
];

export function getTabVisual(tabId: number): TabVisual {
  const index = Math.abs(tabId - 1) % tabPalette.length;
  return tabPalette[index];
}
/** First displayable letter for a channel name, skipping IPTV-style prefixes like "(US)". */
export function channelInitial(name: string): string {
  const cleaned = name.replace(/^[\s\[\(（【「『]+/, "").trim();
  const match = cleaned.match(/[A-Za-z0-9]/);
  return match ? match[0].toUpperCase() : "TV";
}

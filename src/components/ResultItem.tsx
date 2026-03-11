import type { SearchResult } from "../hooks/useSearch";

interface ResultItemProps {
  result: SearchResult;
  selected: boolean;
  index: number;
  onSelect: () => void;
  onHover: () => void;
}

export function ResultItem({
  result,
  selected,
  index,
  onSelect,
  onHover,
}: ResultItemProps) {
  const emoji = KIND_ICONS[result.kind] ?? "📄";
  const shortPath = shortenPath(result.path);

  return (
    <button
      className={`result-item${selected ? " selected" : ""}`}
      onClick={onSelect}
      onMouseEnter={onHover}
      data-index={index}
      role="option"
      aria-selected={selected}
      aria-label={result.title}
    >
      <div className="result-icon" aria-hidden="true">
        {emoji}
      </div>

      <div className="result-text">
        <div className="result-title">{result.title}</div>
        <div className="result-subtitle" title={result.path}>
          {shortPath}
        </div>
      </div>

      <span className="result-kind-badge" aria-label={`Type: ${result.kind}`}>
        {result.kind}
      </span>

      {selected && (
        <div className="result-shortcut" aria-hidden="true">
          <span className="kbd">↵</span>
        </div>
      )}
    </button>
  );
}

const KIND_ICONS: Record<string, string> = {
  application: "🚀",
  document: "📄",
  image: "🖼️",
  video: "🎬",
  audio: "🎵",
  code: "💻",
  folder: "📁",
  file: "📎",
  archive: "📦",
};

function shortenPath(path: string): string {
  const maxLen = 60;
  let p = path;

  // Replace home directory with ~
  const homeMatch = p.match(/^\/(?:home|Users)\/[^/]+\//);
  if (homeMatch) {
    p = "~/" + p.slice(homeMatch[0].length);
  }

  if (p.length <= maxLen) return p;
  const parts = p.split("/");
  if (parts.length > 4) {
    return parts.slice(0, 2).join("/") + "/…/" + parts.slice(-2).join("/");
  }
  return "…" + p.slice(p.length - maxLen);
}

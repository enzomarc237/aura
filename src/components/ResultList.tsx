import { useEffect, useRef, useState } from "react";
import { motion } from "framer-motion";
import { ResultItem } from "./ResultItem";
import type { SearchResult } from "../hooks/useSearch";

interface ResultListProps {
  results: SearchResult[];
  onSelect: (id: number) => void;
}

export function ResultList({ results, onSelect }: ResultListProps) {
  const [selectedIndex, setSelectedIndex] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);

  // Reset selection when results change
  useEffect(() => {
    setSelectedIndex(0);
  }, [results]);

  // Keyboard navigation
  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((i) => Math.min(i + 1, results.length - 1));
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((i) => Math.max(i - 1, 0));
          break;
        case "Enter":
          e.preventDefault();
          if (results[selectedIndex]) {
            onSelect(results[selectedIndex].id);
          }
          break;
        default:
          break;
      }
    };

    document.addEventListener("keydown", handleKey);
    return () => document.removeEventListener("keydown", handleKey);
  }, [results, selectedIndex, onSelect]);

  // Scroll selected item into view
  useEffect(() => {
    const list = listRef.current;
    if (!list) return;
    const item = list.querySelector<HTMLElement>(
      `[data-index="${selectedIndex}"]`,
    );
    item?.scrollIntoView({ block: "nearest", behavior: "smooth" });
  }, [selectedIndex]);

  // Group results by kind
  const grouped = groupBy(results, (r) => r.kind);
  const kinds = Object.keys(grouped);

  let globalIndex = 0;

  return (
    <div
      ref={listRef}
      className="result-list"
      role="listbox"
      aria-label="Search results"
    >
      {kinds.map((kind) => (
        <div key={kind}>
          <div className="result-separator" aria-hidden="true">
            {KIND_LABELS[kind] ?? kind}
          </div>
          {grouped[kind].map((result) => {
            const idx = globalIndex++;
            return (
              <motion.div
                key={result.id}
                initial={{ opacity: 0, x: -8 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ duration: 0.12, delay: idx * 0.025 }}
              >
                <ResultItem
                  result={result}
                  selected={idx === selectedIndex}
                  index={idx}
                  onSelect={() => onSelect(result.id)}
                  onHover={() => setSelectedIndex(idx)}
                />
              </motion.div>
            );
          })}
        </div>
      ))}
    </div>
  );
}

const KIND_LABELS: Record<string, string> = {
  application: "Apps",
  document: "Documents",
  image: "Images",
  video: "Videos",
  audio: "Audio",
  code: "Code",
  folder: "Folders",
  file: "Files",
  archive: "Archives",
};

function groupBy<T>(arr: T[], key: (item: T) => string): Record<string, T[]> {
  return arr.reduce<Record<string, T[]>>((acc, item) => {
    const k = key(item);
    (acc[k] ??= []).push(item);
    return acc;
  }, {});
}

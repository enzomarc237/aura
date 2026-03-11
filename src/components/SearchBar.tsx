import { useEffect, useRef, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
import type { Intent } from "../hooks/useSearch";

const PLACEHOLDER_PHRASES = [
  "Search apps, files, and more…",
  "Try: email Marc Enzo",
  "Try: start pomodoro 25",
  "Try: open Safari",
  "Try: empty trash",
  "Try: search for React docs",
  "Cmd+Space to show · Esc to hide",
];

interface SearchBarProps {
  query: string;
  onQueryChange: (q: string) => void;
  loading: boolean;
  intent: Intent | null;
  onIntentExecute: (action: string, payload: Record<string, unknown>) => void;
}

export function SearchBar({
  query,
  onQueryChange,
  loading,
  intent,
  onIntentExecute,
}: SearchBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [placeholderIdx, setPlaceholderIdx] = useState(0);

  // Cycle through placeholder hints every 3s when query is empty
  useEffect(() => {
    if (query) return;
    const id = setInterval(() => {
      setPlaceholderIdx((i) => (i + 1) % PLACEHOLDER_PHRASES.length);
    }, 3000);
    return () => clearInterval(id);
  }, [query]);

  // Auto-focus input
  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  return (
    <div className="search-bar" role="search" aria-label="Aura search">
      <SearchIcon className="search-icon" loading={loading} />

      <input
        ref={inputRef}
        type="text"
        className={`search-input${loading ? " loading" : ""}`}
        value={query}
        onChange={(e) => onQueryChange(e.target.value)}
        placeholder={PLACEHOLDER_PHRASES[placeholderIdx]}
        autoComplete="off"
        autoCorrect="off"
        autoCapitalize="off"
        spellCheck={false}
        aria-label="Search"
        aria-autocomplete="list"
      />

      <AnimatePresence>
        {intent && (
          <motion.button
            key={intent.action}
            className="intent-badge"
            initial={{ opacity: 0, scale: 0.85 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.85 }}
            transition={{ duration: 0.15 }}
            onClick={() =>
              onIntentExecute(
                intent.action,
                intent.payload as Record<string, unknown>,
              )
            }
            title={`Execute: ${intent.action}`}
            aria-label={`Run intent: ${intent.kind}`}
          >
            <IntentIcon kind={intent.kind} />
            {intentLabel(intent)}
          </motion.button>
        )}
      </AnimatePresence>

      {!query && <span className="search-shortcut">Esc</span>}
    </div>
  );
}

function intentLabel(intent: Intent): string {
  switch (intent.kind) {
    case "email":
      return `Email ${intent.payload.recipient as string}`;
    case "phone":
      return `Call ${intent.payload.contact as string}`;
    case "timer":
      return `Timer ${intent.payload.minutes as number}m`;
    case "web_search":
      return `Search "${intent.payload.query as string}"`;
    case "system":
      return intent.action.replace(/_/g, " ");
    default:
      return intent.action;
  }
}

function IntentIcon({ kind }: { kind: string }) {
  const icons: Record<string, string> = {
    email: "✉️",
    phone: "📞",
    timer: "⏱️",
    web_search: "🌐",
    system: "⚙️",
    open: "↗️",
  };
  return <span aria-hidden="true">{icons[kind] ?? "✨"}</span>;
}

function SearchIcon({
  className,
  loading,
}: {
  className?: string;
  loading: boolean;
}) {
  return (
    <svg
      className={className}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      style={{
        opacity: loading ? 0.5 : 1,
        transition: "opacity 0.2s",
      }}
    >
      <circle cx="11" cy="11" r="8" />
      <line x1="21" y1="21" x2="16.65" y2="16.65" />
    </svg>
  );
}

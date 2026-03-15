import { useEffect, useState } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { SearchBar } from "./components/SearchBar";
import { ResultList } from "./components/ResultList";
import { useSearch } from "./hooks/useSearch";
import "./styles/globals.css";

export default function App() {
  const { query, setQuery, results, intent, loading, executeAction, executeIntent } =
    useSearch();
  const [indexing, setIndexing] = useState(true);
  const [indexCount, setIndexCount] = useState(0);

  useEffect(() => {
    const unlistenComplete = listen<number>("index_complete", (event) => {
      setIndexing(false);
      setIndexCount(event.payload);
    });

    const unlistenError = listen<string>("index_error", () => {
      setIndexing(false);
    });

    return () => {
      unlistenComplete.then((f) => f());
      unlistenError.then((f) => f());
    };
  }, []);

  // Close window on Escape
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        getCurrentWindow().hide();
      }
    };
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, []);

  return (
    <div className="app-root">
      <motion.div
        className="aura-panel"
        initial={{ opacity: 0, y: -20, scale: 0.97 }}
        animate={{ opacity: 1, y: 0, scale: 1 }}
        exit={{ opacity: 0, y: -20, scale: 0.97 }}
        transition={{ duration: 0.18, ease: [0.16, 1, 0.3, 1] }}
      >
        <SearchBar
          query={query}
          onQueryChange={setQuery}
          loading={loading}
          intent={intent}
          onIntentExecute={executeIntent}
        />

        <AnimatePresence mode="sync">
          {results.length > 0 && (
            <motion.div
              key="results"
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: "auto" }}
              exit={{ opacity: 0, height: 0 }}
              transition={{ duration: 0.15, ease: "easeOut" }}
            >
              <ResultList
                results={results}
                onSelect={executeAction}
              />
            </motion.div>
          )}
        </AnimatePresence>

        {indexing && (
          <div className="indexing-status">
            <span className="indexing-dot" />
            Indexing…
          </div>
        )}
        {!indexing && indexCount > 0 && results.length === 0 && !query && (
          <div className="empty-hint">
            {indexCount.toLocaleString()} items indexed · Type to search
          </div>
        )}
      </motion.div>
    </div>
  );
}

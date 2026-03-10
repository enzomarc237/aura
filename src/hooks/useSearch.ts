import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface SearchResult {
  id: number;
  title: string;
  path: string;
  kind: string;
  score: number;
  rank: number;
}

export interface Intent {
  kind: string;
  action: string;
  payload: Record<string, unknown>;
}

interface SearchResponse {
  results: SearchResult[];
  intent: Intent | null;
  query: string;
}

export function useSearch() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [intent, setIntent] = useState<Intent | null>(null);
  const [loading, setLoading] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);

    debounceRef.current = setTimeout(async () => {
      setLoading(true);
      try {
        const response = await invoke<SearchResponse>("search", { query });
        setResults(response.results);
        setIntent(response.intent);
      } catch (err) {
        console.error("Search error:", err);
        setResults([]);
        setIntent(null);
      } finally {
        setLoading(false);
      }
    }, 60);

    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [query]);

  const executeAction = useCallback(
    async (id: number) => {
      try {
        await invoke("execute_action", { id, query });
        setQuery("");
        setResults([]);
      } catch (err) {
        console.error("Execute action error:", err);
      }
    },
    [query],
  );

  const executeIntent = useCallback(
    async (action: string, payload: Record<string, unknown>) => {
      try {
        await invoke("execute_intent", { action, payload });
        setQuery("");
        setResults([]);
        setIntent(null);
      } catch (err) {
        console.error("Execute intent error:", err);
      }
    },
    [],
  );

  return {
    query,
    setQuery,
    results,
    intent,
    loading,
    executeAction,
    executeIntent,
  };
}

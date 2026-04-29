import React, {
  useState,
  useEffect,
  useCallback,
  useRef,
  memo,
  useMemo,
} from "react";
import { useTranslation } from "react-i18next";
import { motion } from "motion/react";
import { toast } from "sonner";
import { staggerContainer, staggerItem } from "@/lib/motion";
import { AudioPlayer } from "../../ui/AudioPlayer";
import { Button } from "../../ui/Button";
import {
  Copy,
  Star,
  Check,
  Trash,
  FolderOpen,
  Microphone,
  Sparkle,
} from "@phosphor-icons/react";
import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { commands, type HistoryEntry } from "@/bindings";
import { useTranscribeShortcut } from "@/hooks/useTranscribeShortcut";
import {
  formatRelativeTime,
  formatAbsoluteTime,
} from "@/lib/utils/relativeTime";
import { SimpleTooltip } from "../../ui/Tooltip";
import { TabBar, type TabItem } from "../../ui/TabBar";
import { RecordingRetentionPeriodSelector } from "../RecordingRetentionPeriod";
import { StatsSettings } from "../stats/StatsSettings";

const PAGE_SIZE = 50;

type HistoryTab = "recordings" | "stats";

interface OpenRecordingsButtonProps {
  onClick: () => void;
  label: string;
}

const OpenRecordingsButton: React.FC<OpenRecordingsButtonProps> = ({
  onClick,
  label,
}) => (
  <Button
    onClick={onClick}
    variant="secondary"
    size="sm"
    className="flex items-center gap-2"
    title={label}
  >
    <FolderOpen className="w-4 h-4" />
    <span>{label}</span>
  </Button>
);

export const HistorySettings: React.FC = () => {
  const { t } = useTranslation();
  const shortcutDisplay = useTranscribeShortcut();
  const [activeTab, setActiveTab] = useState<HistoryTab>("stats");
  const tabs: TabItem[] = useMemo(
    () => [
      { id: "stats", label: t("settings.history.tabs.stats") },
      { id: "recordings", label: t("settings.history.tabs.recordings") },
    ],
    [t],
  );
  const [historyEntries, setHistoryEntries] = useState<HistoryEntry[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [loading, setLoading] = useState(true);
  const [loadingMore, setLoadingMore] = useState(false);
  const observerRef = useRef<IntersectionObserver | null>(null);

  const hasMore = historyEntries.length < totalCount;

  const loadPage = useCallback(
    async (cursor: number | null, replace: boolean) => {
      const result = await commands.getHistoryEntriesPage(PAGE_SIZE, cursor);
      if (result.status === "ok") {
        const { entries, total_count } = result.data;
        if (replace) {
          setHistoryEntries(entries);
        } else {
          setHistoryEntries((prev) => [...prev, ...entries]);
        }
        setTotalCount(total_count);
      }
    },
    [],
  );

  // Initial load
  useEffect(() => {
    loadPage(null, true).finally(() => setLoading(false));
  }, [loadPage]);

  // Listen for new transcriptions from the backend
  useEffect(() => {
    const setupListener = async () => {
      const unlisten = await listen<HistoryEntry>(
        "history-entry-added",
        (event) => {
          setHistoryEntries((prev) => [event.payload, ...prev]);
          setTotalCount((prev) => prev + 1);
        },
      );
      return unlisten;
    };

    const unlistenPromise = setupListener();

    return () => {
      unlistenPromise.then((unlisten) => {
        if (unlisten) {
          unlisten();
        }
      });
    };
  }, []);

  // Infinite scroll via IntersectionObserver using a callback ref.
  // The observer is created when the sentinel mounts (entries loaded)
  // and disconnected when it unmounts (loading/empty states).
  const scrollStateRef = useRef({
    hasMore,
    loadingMore,
    lastId: null as number | null,
  });
  scrollStateRef.current = {
    hasMore,
    loadingMore,
    lastId: historyEntries[historyEntries.length - 1]?.id ?? null,
  };

  const sentinelRef = useCallback(
    (node: HTMLDivElement | null) => {
      if (observerRef.current) {
        observerRef.current.disconnect();
        observerRef.current = null;
      }
      if (node) {
        observerRef.current = new IntersectionObserver(
          ([entry]) => {
            const { hasMore, loadingMore, lastId } = scrollStateRef.current;
            if (entry.isIntersecting && hasMore && !loadingMore) {
              setLoadingMore(true);
              loadPage(lastId, false).finally(() => setLoadingMore(false));
            }
          },
          { rootMargin: "200px" },
        );
        observerRef.current.observe(node);
      }
    },
    [loadPage],
  );

  const toggleSaved = useCallback(async (id: number) => {
    setHistoryEntries((prev) =>
      prev.map((e) => (e.id === id ? { ...e, saved: !e.saved } : e)),
    );
    try {
      await commands.toggleHistoryEntrySaved(id);
    } catch (error) {
      console.error("Failed to toggle saved status:", error);
      // Revert on error
      setHistoryEntries((prev) =>
        prev.map((e) => (e.id === id ? { ...e, saved: !e.saved } : e)),
      );
    }
  }, []);

  const copyToClipboard = useCallback(async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch (error) {
      console.error("Failed to copy to clipboard:", error);
    }
  }, []);

  const getAudioUrl = useCallback(async (fileName: string) => {
    try {
      const result = await commands.getAudioFilePath(fileName);
      if (result.status === "ok") {
        return convertFileSrc(result.data, "asset");
      }
      return null;
    } catch (error) {
      console.error("Failed to get audio file path:", error);
      return null;
    }
  }, []);

  const deleteEntry = useCallback(
    async (id: number) => {
      setHistoryEntries((prev) => prev.filter((e) => e.id !== id));
      setTotalCount((prev) => prev - 1);
      try {
        await commands.deleteHistoryEntry(id);
      } catch (error) {
        console.error("Failed to delete entry:", error);
        // Reload on error to restore correct state
        loadPage(null, true);
      }
    },
    [loadPage],
  );

  const openRecordingsFolder = async () => {
    try {
      await commands.openRecordingsFolder();
    } catch (error) {
      console.error("Failed to open recordings folder:", error);
    }
  };

  const retentionSection = (
    <RecordingRetentionPeriodSelector
      descriptionMode="tooltip"
      grouped={false}
    />
  );

  let recordingsContent;
  if (loading) {
    recordingsContent = (
      <div className="bg-background-translucent border border-glass-border rounded overflow-visible">
        <div className="px-3 py-8 flex flex-col items-center gap-3">
          <div className="w-5 h-5 border-2 border-muted/40 border-t-accent rounded-full animate-spin" />
          <p className="text-sm text-muted">{t("settings.history.loading")}</p>
        </div>
      </div>
    );
  } else if (historyEntries.length === 0) {
    recordingsContent = (
      <div className="bg-background-translucent border border-glass-border rounded overflow-visible">
        <div className="px-3 py-10 flex flex-col items-center gap-3">
          <div className="w-10 h-10 rounded-full bg-muted/10 flex items-center justify-center">
            <Microphone className="w-5 h-5 text-muted/60" />
          </div>
          <div className="flex flex-col items-center gap-1">
            <p className="text-sm text-muted text-center">
              {t("settings.history.empty", { shortcut: shortcutDisplay })}
            </p>
            <p className="text-xs text-muted/60 text-center">
              {t("settings.history.emptyHint")}
            </p>
          </div>
        </div>
      </div>
    );
  } else {
    recordingsContent = (
      <div className="flex flex-col gap-2">
        {historyEntries.map((entry) => (
          <HistoryEntryComponent
            key={entry.id}
            entry={entry}
            onToggleSaved={toggleSaved}
            onCopy={copyToClipboard}
            getAudioUrl={getAudioUrl}
            onDelete={deleteEntry}
          />
        ))}
        <div ref={sentinelRef} className="py-4 flex justify-center">
          {loadingMore && (
            <div className="w-5 h-5 border-2 border-muted/40 border-t-accent rounded-full animate-spin" />
          )}
        </div>
      </div>
    );
  }

  return (
    <motion.div
      className="max-w-3xl w-full space-y-8"
      variants={staggerContainer}
      initial="initial"
      animate="animate"
    >
      <h1 className="sr-only">{t("sidebar.history")}</h1>

      <motion.div variants={staggerItem} style={{ willChange: "transform" }}>
        <TabBar
          tabs={tabs}
          activeTab={activeTab}
          onTabChange={(id) => setActiveTab(id as HistoryTab)}
        />
      </motion.div>

      <motion.div variants={staggerItem} style={{ willChange: "transform" }}>
        <div
          role="tabpanel"
          id="tabpanel-stats"
          aria-labelledby="tab-stats"
          className={activeTab !== "stats" ? "hidden" : undefined}
        >
          {activeTab === "stats" && <StatsSettings />}
        </div>
        <div
          role="tabpanel"
          id="tabpanel-recordings"
          aria-labelledby="tab-recordings"
          className={activeTab !== "recordings" ? "hidden" : undefined}
        >
          <div className="space-y-8">
            {retentionSection}
            <div className="space-y-1.5">
              <div className="px-3 flex items-center justify-between">
                <div className="flex items-center gap-2">
                  <h2 className="text-xs font-medium text-muted uppercase tracking-wide">
                    {t("settings.history.title")}
                  </h2>
                  {!loading && totalCount > 0 && (
                    <span className="text-xs text-muted/50 tabular-nums">
                      {totalCount}
                    </span>
                  )}
                </div>
                <OpenRecordingsButton
                  onClick={openRecordingsFolder}
                  label={t("settings.history.openFolder")}
                />
              </div>
              {recordingsContent}
            </div>
          </div>
        </div>
      </motion.div>
    </motion.div>
  );
};

interface HistoryEntryProps {
  entry: HistoryEntry;
  onToggleSaved: (id: number) => void;
  onCopy: (text: string) => void;
  getAudioUrl: (fileName: string) => Promise<string | null>;
  onDelete: (id: number) => void;
}

const HistoryEntryComponent: React.FC<HistoryEntryProps> = memo(
  ({ entry, onToggleSaved, onCopy, getAudioUrl, onDelete }) => {
    const { t, i18n } = useTranslation();
    const [showCopied, setShowCopied] = useState(false);
    const [expanded, setExpanded] = useState(false);

    const handleLoadAudio = useCallback(
      () => getAudioUrl(entry.file_name),
      [getAudioUrl, entry.file_name],
    );

    const displayText = entry.post_processed_text || entry.transcription_text;
    const hasPostProcessed = !!entry.post_processed_text;

    const handleCopyText = () => {
      onCopy(displayText);
      setShowCopied(true);
      setTimeout(() => setShowCopied(false), 2000);
    };

    const handleDeleteEntry = () => {
      onDelete(entry.id);
    };

    const ts = Number(entry.timestamp);
    const relativeTime = useMemo(
      () => formatRelativeTime(ts, i18n.language),
      [ts, i18n.language],
    );
    const absoluteTime = useMemo(() => formatAbsoluteTime(ts), [ts]);

    return (
      <div className="group relative bg-background-translucent border border-glass-border rounded hover:border-glass-border-hover transition-colors px-3 py-2 flex flex-col gap-1.5">
        {/* Action buttons - absolutely positioned top-right */}
        <div className="absolute top-1.5 right-1.5 flex items-center gap-0.5">
          {/* Saved star - always visible when saved */}
          {entry.saved && (
            <SimpleTooltip content={t("settings.history.unsave")}>
              <button
                onClick={() => onToggleSaved(entry.id)}
                className="p-2 min-w-[36px] min-h-[36px] flex items-center justify-center rounded text-accent hover:text-accent/80 transition-colors cursor-pointer"
              >
                <Star size={14} weight="fill" />
              </button>
            </SimpleTooltip>
          )}
          {/* Other actions - visible on hover */}
          <div className="flex items-center gap-0.5 opacity-30 group-hover:opacity-100 group-focus-within:opacity-100 transition-opacity">
            <SimpleTooltip content={t("settings.history.copyToClipboard")}>
              <button
                onClick={handleCopyText}
                className="p-2 min-w-[36px] min-h-[36px] flex items-center justify-center rounded text-text/50 hover:text-accent transition-colors cursor-pointer"
              >
                {showCopied ? <Check size={14} /> : <Copy size={14} />}
              </button>
            </SimpleTooltip>
            {!entry.saved && (
              <SimpleTooltip content={t("settings.history.save")}>
                <button
                  onClick={() => onToggleSaved(entry.id)}
                  className="p-2 min-w-[36px] min-h-[36px] flex items-center justify-center rounded text-text/50 hover:text-accent transition-colors cursor-pointer"
                >
                  <Star size={14} weight="light" />
                </button>
              </SimpleTooltip>
            )}
            <SimpleTooltip content={t("settings.history.delete")}>
              <button
                onClick={handleDeleteEntry}
                className="p-2 min-w-[36px] min-h-[36px] flex items-center justify-center rounded text-text/50 hover:text-error transition-colors cursor-pointer"
              >
                <Trash size={14} />
              </button>
            </SimpleTooltip>
          </div>
        </div>

        {/* Text content */}
        <p className="text-[13px] leading-snug text-text/90 select-text cursor-text pr-28">
          {displayText}
        </p>
        {hasPostProcessed && (
          <button
            onClick={() => setExpanded(!expanded)}
            className="flex items-center gap-1 text-xs text-muted/60 hover:text-muted transition-colors cursor-pointer"
          >
            <Sparkle size={10} />
            <span>
              {expanded
                ? t("settings.history.hideOriginal")
                : t("settings.history.showOriginal")}
            </span>
          </button>
        )}
        {hasPostProcessed && expanded && (
          <motion.p
            initial={{ opacity: 0, y: -4 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.15, ease: [0.16, 1, 0.3, 1] }}
            className="text-[12px] leading-snug text-muted/70 select-text cursor-text border-l-2 border-glass-border pl-2"
          >
            {entry.transcription_text}
          </motion.p>
        )}

        {/* Audio player + timestamp */}
        <div className="flex items-center gap-2">
          <AudioPlayer onLoadRequest={handleLoadAudio} className="flex-1" />
          <SimpleTooltip content={absoluteTime}>
            <span className="text-xs text-muted/80 whitespace-nowrap shrink-0 tabular-nums cursor-default">
              {relativeTime}
            </span>
          </SimpleTooltip>
        </div>
      </div>
    );
  },
);

HistoryEntryComponent.displayName = "HistoryEntryComponent";

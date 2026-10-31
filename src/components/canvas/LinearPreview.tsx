import { memo, useEffect, useRef } from 'react';
import { VideoInfo } from '../../types';
import { VideoCard } from './VideoCard';

export interface LinearPreviewProps {
  items: VideoInfo[];
  viewMode: 'grid' | 'list';
  onReachBoundary?: () => void;
}

/**
 * 线性预览（固定网格 / 列表）。
 * 与无限画布不同，这里使用标准的滚动容器 + 底部哨兵触发分页，
 * 卡片为统一尺寸，方便快速浏览大量结果。
 */
export const LinearPreview = memo(function LinearPreview({
  items,
  viewMode,
  onReachBoundary,
}: LinearPreviewProps) {
  const sentinelRef = useRef<HTMLDivElement | null>(null);
  const cooldownRef = useRef(0);

  useEffect(() => {
    const sentinel = sentinelRef.current;
    if (!sentinel || !onReachBoundary) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (!entries.some((e) => e.isIntersecting)) return;
        const now = Date.now();
        // 冷却 1200ms，避免在分页加载尚未完成时连续触发
        if (now - cooldownRef.current < 1200) return;
        cooldownRef.current = now;
        onReachBoundary();
      },
      { rootMargin: '700px 0px 0px 0px' }
    );

    observer.observe(sentinel);
    return () => observer.disconnect();
  }, [onReachBoundary]);

  return (
    <div className="h-screen w-screen overflow-y-auto overflow-x-hidden bg-black [scrollbar-width:thin]">
      {viewMode === 'grid' ? (
        <div className="grid grid-flow-row gap-2 p-4 pb-24 [grid-template-columns:repeat(auto-fill,minmax(230px,1fr))]">
          {items.map((card) => (
            <div key={card.id} className="aspect-[16/10]">
              <VideoCard data={card} />
            </div>
          ))}
        </div>
      ) : (
        <div className="mx-auto flex w-full max-w-3xl flex-col gap-2 p-4 pb-24">
          {items.map((card) => (
            <div key={card.id} className="flex items-stretch gap-3 rounded-lg bg-surface-card p-2 border border-subtle">
              <div className="aspect-video w-60 shrink-0">
                <VideoCard data={card} />
              </div>
              <div className="flex min-w-0 flex-1 flex-col justify-center gap-1.5 pl-1">
                <span className="text-card-title line-clamp-2">{card.title || '—'}</span>
                {card.duration ? (
                  <span className="text-meta">{card.duration}</span>
                ) : null}
              </div>
            </div>
          ))}
        </div>
      )}
      <div ref={sentinelRef} className="h-px w-full" aria-hidden />
    </div>
  );
});

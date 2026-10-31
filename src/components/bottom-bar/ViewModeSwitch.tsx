import { useTranslation } from 'react-i18next';
import { LayoutGrid, List, Move } from 'lucide-react';
import { useUIStore, type ViewMode } from '../../store/uiStore';
import { cn } from '../../utils/cn';

const MODES: { value: ViewMode; icon: typeof Move; titleKey: string }[] = [
  { value: 'canvas', icon: Move, titleKey: 'bottomBar.viewMode.canvas' },
  { value: 'grid', icon: LayoutGrid, titleKey: 'bottomBar.viewMode.grid' },
  { value: 'list', icon: List, titleKey: 'bottomBar.viewMode.list' },
];

/**
 * 预览模式切换器：画布（可变尺寸卡片） / 固定网格 / 列表
 */
export function ViewModeSwitch() {
  const { t } = useTranslation();
  const viewMode = useUIStore((s) => s.viewMode);
  const setViewMode = useUIStore((s) => s.setViewMode);

  return (
    <div className="flex items-center gap-0.5 rounded-lg bg-surface-subtle p-0.5">
      {MODES.map(({ value, icon: Icon, titleKey }) => {
        const active = viewMode === value;
        return (
          <button
            key={value}
            type="button"
            onClick={() => setViewMode(value)}
            title={t(titleKey)}
            aria-label={t(titleKey)}
            className={cn(
              'flex size-7 items-center justify-center rounded-md transition cursor-pointer',
              active
                ? 'bg-white/15 text-white'
                : 'text-white/45 hover:bg-white/10 hover:text-white'
            )}
          >
            <Icon size={14} strokeWidth={1.8} />
          </button>
        );
      })}
    </div>
  );
}

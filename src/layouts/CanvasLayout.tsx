import { useEffect } from 'react';
import { BottomBar } from '../components/bottom-bar/BottomBar';
import { ModalHost } from '../components/modals/ModalHost';
import { useDownloadStore } from '../store/downloadStore';
import { useUpdateStore } from '../store/updateStore';
import { useUIStore } from '../store/uiStore';
import { useAppConfig, useVideoFeed } from '../hooks/queries';
import { Toast } from '../components/ui/Toast';
import { Loading } from '../components/ui/Loading';
import { WindowDragBar } from '../components/ui/WindowDragBar';
import { cn } from '../utils/cn';
import { InfiniteCanvas } from '../components/canvas/InfiniteCanvas';
import { LinearPreview } from '../components/canvas/LinearPreview';

export function CanvasLayout() {
  useAppConfig();
  const { cards, fetchNexts } = useVideoFeed();
  const isVideoLoading = useUIStore((s) => s.isVideoLoading);
  const viewMode = useUIStore((s) => s.viewMode);

  useEffect(() => {
    useDownloadStore.getState().initListeners();
    useUpdateStore.getState().initAutoCheck();
  }, []);

  return (
    <div className={cn('app-viewport')}>
      <WindowDragBar />
      <Toast />

      {viewMode === 'canvas' ? (
        <InfiniteCanvas items={cards} onReachBoundary={fetchNexts} />
      ) : (
        <LinearPreview items={cards} viewMode={viewMode} onReachBoundary={fetchNexts} />
      )}

      {isVideoLoading && cards.length === 0 && <Loading />}

      <BottomBar />

      <ModalHost />
    </div>
  );
}

export default CanvasLayout;

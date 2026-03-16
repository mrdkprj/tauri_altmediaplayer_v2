import Deferred from "./deferred";

// Linux only
type ContextMenuState = {
    deferred: Deferred<number> | null;
};
const contextMenuState: ContextMenuState = $state({ deferred: null });
export const awaitContextMenu = async () => {
    contextMenuState.deferred = new Deferred();
    await contextMenuState.deferred.promise;
};
export const resolveContextMenu = () => {
    if (contextMenuState.deferred) {
        contextMenuState.deferred.resolve(0);
        contextMenuState.deferred = null;
    }
};

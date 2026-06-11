/**
 * VM Bridge service for Sailfish Studio editor.
 * Communicates with sf-vm via Web Worker interface.
 * Provides Promise-based API for VM operations and event listener system.
 */

/** Event types emitted by the VM */
export type VmEventType = 'started' | 'stopped' | 'compiled' | 'broadcast' | 'error' | 'variable-changed';

/** Event listener callback */
export type VmEventListener = (data: unknown) => void;

/** Result from VM operations */
export interface VmResult<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
}

/**
 * VmBridge communicates with the sf-vm runtime via a Web Worker.
 * All methods return Promises and an event system allows listening for VM events.
 */
export class VmBridge {
  private worker: Worker | null = null;
  private listeners: Map<VmEventType, Set<VmEventListener>> = new Map();
  private pendingRequests: Map<
    number,
    { resolve: (value: VmResult) => void; reject: (reason: unknown) => void }
  > = new Map();
  private nextRequestId = 0;

  /** Connect to a VM web worker */
  connect(worker: Worker): void {
    this.worker = worker;
    this.worker.onmessage = (event: MessageEvent) => {
      this.handleMessage(event.data);
    };
    this.worker.onerror = (event: ErrorEvent) => {
      this.emit('error', { message: event.message });
    };
  }

  /** Disconnect from the VM worker */
  disconnect(): void {
    if (this.worker) {
      this.worker.onmessage = null;
      this.worker.onerror = null;
      this.worker = null;
    }
    // Reject all pending requests
    for (const [id, pending] of this.pendingRequests) {
      pending.reject(new Error('VM disconnected'));
    }
    this.pendingRequests.clear();
  }

  /** Load a project into the VM from a JSON string */
  loadProject(json: string): Promise<VmResult> {
    return this.sendRequest('loadProject', { json });
  }

  /** Compile the current project */
  compile(): Promise<VmResult> {
    return this.sendRequest('compile', {});
  }

  /** Start the VM */
  start(): Promise<VmResult> {
    return this.sendRequest('start', {});
  }

  /** Stop the VM */
  stop(): Promise<VmResult> {
    return this.sendRequest('stop', {});
  }

  /** Broadcast a message to the VM */
  broadcast(name: string): Promise<VmResult> {
    return this.sendRequest('broadcast', { name });
  }

  /** Get a variable value from the VM */
  getVariable(name: string): Promise<VmResult> {
    return this.sendRequest('getVariable', { name });
  }

  /** Add an event listener */
  on(event: VmEventType, listener: VmEventListener): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(listener);
  }

  /** Remove an event listener */
  off(event: VmEventType, listener: VmEventListener): void {
    this.listeners.get(event)?.delete(listener);
  }

  /** Emit an event to all registered listeners */
  emit(event: VmEventType, data: unknown): void {
    this.listeners.get(event)?.forEach((listener) => {
      try {
        listener(data);
      } catch {
        // Swallow listener errors
      }
    });
  }

  /** Check if connected to a worker */
  isConnected(): boolean {
    return this.worker !== null;
  }

  /** Send a request to the VM worker and return a promise */
  private sendRequest(method: string, params: Record<string, unknown>): Promise<VmResult> {
    if (!this.worker) {
      return Promise.resolve({
        success: false,
        error: 'VM not connected',
      });
    }

    const id = this.nextRequestId++;
    const promise = new Promise<VmResult>((resolve, reject) => {
      this.pendingRequests.set(id, { resolve, reject });
    });

    this.worker.postMessage({ id, method, params });

    return promise;
  }

  /** Handle incoming message from the VM worker */
  private handleMessage(data: { id?: number; event?: VmEventType; result?: VmResult; data?: unknown }): void {
    // Handle response to a request
    if (data.id !== undefined && this.pendingRequests.has(data.id)) {
      const pending = this.pendingRequests.get(data.id)!;
      this.pendingRequests.delete(data.id);
      pending.resolve(data.result ?? { success: true, data: data.data });
    }

    // Handle event notification from VM
    if (data.event) {
      this.emit(data.event, data.data);
    }
  }
}

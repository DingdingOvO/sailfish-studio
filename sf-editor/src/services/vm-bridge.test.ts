import { describe, it, expect, vi, beforeEach } from 'vitest';
import { VmBridge, type VmResult } from './vm-bridge';

/**
 * Helper to create a mock Worker that can respond to messages.
 */
function createMockWorker() {
  let onmessage: ((event: MessageEvent) => void) | null = null;
  let onerror: ((event: ErrorEvent) => void) | null = null;

  const postMessage = vi.fn();
  const terminate = vi.fn();

  // Simulate the worker sending a response
  const simulateResponse = (id: number, result: VmResult) => {
    if (onmessage) {
      onmessage(new MessageEvent('message', { data: { id, result } }));
    }
  };

  // Simulate the worker sending an event
  const simulateEvent = (event: string, data: unknown) => {
    if (onmessage) {
      onmessage(new MessageEvent('message', { data: { event, data } }));
    }
  };

  return {
    onmessage,
    onerror,
    postMessage,
    terminate,
    set onmessageHandler(handler: (event: MessageEvent) => void) {
      onmessage = handler;
    },
    set onerrorHandler(handler: (event: ErrorEvent) => void) {
      onerror = handler;
    },
    simulateResponse,
    simulateEvent,
  };
}

describe('VmBridge', () => {
  let bridge: VmBridge;
  let mockWorker: ReturnType<typeof createMockWorker>;

  beforeEach(() => {
    bridge = new VmBridge();
    mockWorker = createMockWorker();

    // Wire up the bridge's connect to use our mock
    // We simulate the connect by directly setting the worker and handlers
    const workerProxy = {
      get onmessage() { return null; },
      set onmessage(handler: ((event: MessageEvent) => void) | null) {
        mockWorker.onmessageHandler = handler!;
      },
      get onerror() { return null; },
      set onerror(handler: ((event: ErrorEvent) => void) | null) {
        mockWorker.onerrorHandler = handler!;
      },
      postMessage: mockWorker.postMessage,
      terminate: mockWorker.terminate,
    } as unknown as Worker;

    bridge.connect(workerProxy);
  });

  it('should connect to a worker', () => {
    expect(bridge.isConnected()).toBe(true);
  });

  it('should disconnect from a worker', () => {
    bridge.disconnect();
    expect(bridge.isConnected()).toBe(false);
  });

  it('should send loadProject request and receive response', async () => {
    const promise = bridge.loadProject('{"targets":[]}');

    // The postMessage should have been called
    expect(mockWorker.postMessage).toHaveBeenCalled();
    const call = mockWorker.postMessage.mock.calls[0][0];
    expect(call.method).toBe('loadProject');
    expect(call.params.json).toBe('{"targets":[]}');

    // Simulate response
    mockWorker.simulateResponse(call.id, { success: true });

    const result = await promise;
    expect(result.success).toBe(true);
  });

  it('should return error when not connected', async () => {
    bridge.disconnect();
    const result = await bridge.loadProject('{}');
    expect(result.success).toBe(false);
    expect(result.error).toBe('VM not connected');
  });

  it('should handle event notifications from VM', () => {
    const listener = vi.fn();
    bridge.on('started', listener);

    mockWorker.simulateEvent('started', { timestamp: Date.now() });

    expect(listener).toHaveBeenCalledOnce();
    expect(listener).toHaveBeenCalledWith({ timestamp: expect.any(Number) });
  });

  it('should support removing event listeners', () => {
    const listener = vi.fn();
    bridge.on('stopped', listener);
    bridge.off('stopped', listener);

    mockWorker.simulateEvent('stopped', null);

    expect(listener).not.toHaveBeenCalled();
  });

  it('should send compile request', async () => {
    const promise = bridge.compile();

    const call = mockWorker.postMessage.mock.calls[0][0];
    expect(call.method).toBe('compile');

    mockWorker.simulateResponse(call.id, { success: true });
    const result = await promise;
    expect(result.success).toBe(true);
  });

  it('should send start and stop requests', async () => {
    const startPromise = bridge.start();
    const startCall = mockWorker.postMessage.mock.calls[0][0];
    expect(startCall.method).toBe('start');
    mockWorker.simulateResponse(startCall.id, { success: true });

    const startResult = await startPromise;
    expect(startResult.success).toBe(true);

    const stopPromise = bridge.stop();
    const stopCall = mockWorker.postMessage.mock.calls[1][0];
    expect(stopCall.method).toBe('stop');
    mockWorker.simulateResponse(stopCall.id, { success: true });

    const stopResult = await stopPromise;
    expect(stopResult.success).toBe(true);
  });

  it('should send broadcast request with name', async () => {
    const promise = bridge.broadcast('hello');

    const call = mockWorker.postMessage.mock.calls[0][0];
    expect(call.method).toBe('broadcast');
    expect(call.params.name).toBe('hello');

    mockWorker.simulateResponse(call.id, { success: true });
    const result = await promise;
    expect(result.success).toBe(true);
  });

  it('should send getVariable request', async () => {
    const promise = bridge.getVariable('score');

    const call = mockWorker.postMessage.mock.calls[0][0];
    expect(call.method).toBe('getVariable');
    expect(call.params.name).toBe('score');

    mockWorker.simulateResponse(call.id, { success: true, data: 42 });
    const result = await promise;
    expect(result.success).toBe(true);
    expect(result.data).toBe(42);
  });

  it('should reject pending requests on disconnect', async () => {
    const promise = bridge.start();

    bridge.disconnect();

    await expect(promise).rejects.toThrow('VM disconnected');
  });
});

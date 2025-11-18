import type { MBOMsgEffect } from '$lib/types';

export type StreamStatus = 'idle' | 'connecting' | 'streaming' | 'complete' | 'error';

export interface StreamProgress {
  loaded: number;
  status: StreamStatus;
  error?: string;
}

export class MarketStreamClient {
  private eventSource: EventSource | null = null;
  private messages: MBOMsgEffect[] = [];
  private onProgressCallback?: (progress: StreamProgress) => void;
  private onMessageCallback?: (message: MBOMsgEffect) => void;
  private onCompleteCallback?: (messages: MBOMsgEffect[]) => void;

  constructor() {}

  /**
   * Start streaming MBO messages from the backend
   */
  async start(
    onProgress?: (progress: StreamProgress) => void,
    onMessage?: (message: MBOMsgEffect) => void,
    onComplete?: (messages: MBOMsgEffect[]) => void
  ): Promise<void> {
    this.onProgressCallback = onProgress;
    this.onMessageCallback = onMessage;
    this.onCompleteCallback = onComplete;
    this.messages = [];

    this.notifyProgress({ loaded: 0, status: 'connecting' });

    return new Promise((resolve, reject) => {
      try {
        this.eventSource = new EventSource('/api/mbo/stream/json/10');

        this.eventSource.onopen = () => {
          console.log('SSE connection opened');
          this.notifyProgress({ loaded: 0, status: 'streaming' });
        };

        this.eventSource.onmessage = (event) => {
          // Check for stream end comment
          if (event.data === '') return; // Keep-alive or comment
          
          try {
            const msgEffect: MBOMsgEffect = JSON.parse(event.data);
            this.messages.push(msgEffect);
            
            this.notifyProgress({ 
              loaded: this.messages.length, 
              status: 'streaming' 
            });

            if (this.onMessageCallback) {
              this.onMessageCallback(msgEffect);
            }
          } catch (err) {
            console.error('Failed to parse message:', err);
          }
        };

        this.eventSource.onerror = (error) => {
          console.log('SSE connection closed or error occurred');
          
          // EventSource automatically closes on error
          if (this.eventSource?.readyState === EventSource.CLOSED) {
            // Stream completed successfully
            this.notifyProgress({ 
              loaded: this.messages.length, 
              status: 'complete' 
            });

            if (this.onCompleteCallback) {
              this.onCompleteCallback([...this.messages]);
            }

            this.cleanup();
            resolve();
          } else {
            // Actual error
            this.notifyProgress({ 
              loaded: this.messages.length, 
              status: 'error',
              error: 'Stream connection error'
            });
            this.cleanup();
            reject(error);
          }
        };

      } catch (err) {
        this.notifyProgress({ 
          loaded: 0, 
          status: 'error',
          error: err instanceof Error ? err.message : 'Unknown error'
        });
        reject(err);
      }
    });
  }

  /**
   * Stop streaming and cleanup
   */
  stop(): void {
    this.cleanup();
  }

  /**
   * Get all loaded messages
   */
  getMessages(): MBOMsgEffect[] {
    return [...this.messages];
  }

  private notifyProgress(progress: StreamProgress): void {
    if (this.onProgressCallback) {
      this.onProgressCallback(progress);
    }
  }

  private cleanup(): void {
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
    }
  }
}

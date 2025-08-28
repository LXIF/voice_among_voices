export class MultiBufferPlayer {
    private audioContext: AudioContext;
    private buffers: AudioBuffer[] = [];
    private currentBufferIndex: number = 0;
    private currentSource: AudioBufferSourceNode | null = null;
    private isPlaying: boolean = false;
    private startTime: number = 0;
    private pausedTime: number = 0;
    private totalDuration: number = 240; // 4 minutes in seconds
    private bufferDuration: number = 30; // 30 seconds per buffer
    private sampleRate: number = 44100; // Standard sample rate
    private channels: number = 2; // Stereo
    private onBufferComplete: (() => void) | null = null;
    private onPlaybackComplete: (() => void) | null = null;
  
    constructor() {
      this.audioContext = new AudioContext();
      this.initializeBuffers();
    }
  
    private initializeBuffers(): void {
      const numBuffers = Math.ceil(this.totalDuration / this.bufferDuration);
      
      for (let i = 0; i < numBuffers; i++) {
        const bufferLength = this.bufferDuration * this.sampleRate;
        const buffer = this.audioContext.createBuffer(
          this.channels,
          bufferLength,
          this.sampleRate
        );
        
        // Initialize with silence
        for (let channel = 0; channel < this.channels; channel++) {
          const channelData = buffer.getChannelData(channel);
          channelData.fill(0);
        }
        
        this.buffers.push(buffer);
      }
    }
  
    /**
     * Add PCM data to a specific buffer at a specific offset
     * @param bufferIndex - Index of the buffer to add data to
     * @param offset - Offset in samples within the buffer
     * @param pcmData - Raw PCM data as Float32Array
     * @param channel - Channel index (0 for left, 1 for right)
     */
    addPCMData(bufferIndex: number, offset: number, pcmData: Float32Array, channel: number): void {
      if (bufferIndex >= this.buffers.length || channel >= this.channels) {
        throw new Error('Invalid buffer index or channel');
      }
  
      const buffer = this.buffers[bufferIndex];
      const channelData = buffer.getChannelData(channel);
      
      // Ensure we don't write beyond the buffer's capacity
      const maxSamples = Math.min(pcmData.length, channelData.length - offset);
      
      for (let i = 0; i < maxSamples; i++) {
        channelData[offset + i] = pcmData[i];
      }
    }
  
    /**
     * Append PCM data sequentially across buffers
     * @param pcmData - Raw PCM data as Float32Array
     * @param channel - Channel index (0 for left, 1 for right)
     */
    appendPCMData(pcmData: Float32Array, channel: number): void {
      let remainingSamples = pcmData.length;
      let dataIndex = 0;
      let currentBufferIndex = 0;
      let currentOffset = 0;
  
      while (remainingSamples > 0 && currentBufferIndex < this.buffers.length) {
        const buffer = this.buffers[currentBufferIndex];
        const channelData = buffer.getChannelData(channel);
        const availableSpace = channelData.length - currentOffset;
        const samplesToWrite = Math.min(remainingSamples, availableSpace);
  
        for (let i = 0; i < samplesToWrite; i++) {
          channelData[currentOffset + i] = pcmData[dataIndex + i];
        }
  
        dataIndex += samplesToWrite;
        remainingSamples -= samplesToWrite;
        currentOffset += samplesToWrite;
  
        if (currentOffset >= channelData.length) {
          currentBufferIndex++;
          currentOffset = 0;
        }
      }
    }
  
    /**
     * Play audio from a specific time
     * @param startTimeSeconds - Time to start playing from (in seconds)
     */
    playFromTime(startTimeSeconds: number): void {
      if (startTimeSeconds < 0 || startTimeSeconds >= this.totalDuration) {
        throw new Error('Start time out of range');
      }
  
      this.stop();
      this.startTime = startTimeSeconds;
      this.pausedTime = startTimeSeconds;
      
      const bufferIndex = Math.floor(startTimeSeconds / this.bufferDuration);
      const offsetInBuffer = (startTimeSeconds % this.bufferDuration) * this.sampleRate;
      
      this.currentBufferIndex = bufferIndex;
      this.playCurrentBuffer(offsetInBuffer);
    }
  
    private playCurrentBuffer(offsetInSamples: number = 0): void {
      if (this.currentBufferIndex >= this.buffers.length) {
        this.onPlaybackComplete?.();
        return;
      }
  
      const buffer = this.buffers[this.currentBufferIndex];
      this.currentSource = this.audioContext.createBufferSource();
      this.currentSource.buffer = buffer;
      this.currentSource.connect(this.audioContext.destination);
  
      // Calculate the actual start time within the buffer
      const startTimeInBuffer = offsetInSamples / this.sampleRate;
      
      this.currentSource.onended = () => {
        this.onBufferComplete?.();
        this.playNextBuffer();
      };
  
      // Schedule playback
      const startTime = this.audioContext.currentTime;
      this.currentSource.start(startTime, startTimeInBuffer);
      this.isPlaying = true;
    }
  
    private playNextBuffer(): void {
      this.currentBufferIndex++;
      if (this.currentBufferIndex < this.buffers.length) {
        this.playCurrentBuffer();
      } else {
        this.isPlaying = false;
        this.onPlaybackComplete?.();
      }
    }
  
    /**
     * Play audio from the beginning
     */
    play(): void {
      this.playFromTime(0);
    }
  
    /**
     * Resume playback from where it was paused
     */
    resume(): void {
      if (!this.isPlaying) {
        this.playFromTime(this.pausedTime);
      }
    }
  
    /**
     * Pause playback
     */
    pause(): void {
      if (this.isPlaying && this.currentSource) {
        this.currentSource.stop();
        this.isPlaying = false;
        
        // Calculate current position
        const currentTime = this.audioContext.currentTime;
        const bufferStartTime = this.startTime + (this.currentBufferIndex * this.bufferDuration);
        const timeInCurrentBuffer = currentTime - bufferStartTime;
        this.pausedTime = bufferStartTime + timeInCurrentBuffer;
      }
    }
  
    /**
     * Stop playback and reset to beginning
     */
    stop(): void {
      if (this.currentSource) {
        this.currentSource.stop();
        this.currentSource = null;
      }
      this.isPlaying = false;
      this.currentBufferIndex = 0;
      this.startTime = 0;
      this.pausedTime = 0;
    }
  
    /**
     * Seek to a specific time
     * @param timeSeconds - Time to seek to (in seconds)
     */
    seek(timeSeconds: number): void {
      if (this.isPlaying) {
        this.playFromTime(timeSeconds);
      } else {
        this.pausedTime = timeSeconds;
      }
    }
  
    /**
     * Get current playback time
     */
    getCurrentTime(): number {
      if (!this.isPlaying) {
        return this.pausedTime;
      }
  
      const bufferStartTime = this.startTime + (this.currentBufferIndex * this.bufferDuration);
      const currentTime = this.audioContext.currentTime;
      return bufferStartTime + (currentTime - bufferStartTime);
    }
  
    /**
     * Get total duration
     */
    getDuration(): number {
      return this.totalDuration;
    }
  
    /**
     * Check if currently playing
     */
    isCurrentlyPlaying(): boolean {
      return this.isPlaying;
    }
  
    /**
     * Set callback for when a buffer completes
     */
    setOnBufferComplete(callback: () => void): void {
      this.onBufferComplete = callback;
    }
  
    /**
     * Set callback for when playback completes
     */
    setOnPlaybackComplete(callback: () => void): void {
      this.onPlaybackComplete = callback;
    }
  
    /**
     * Get the number of buffers
     */
    getBufferCount(): number {
      return this.buffers.length;
    }
  
    /**
     * Get buffer duration
     */
    getBufferDuration(): number {
      return this.bufferDuration;
    }
  
    /**
     * Clean up resources
     */
    dispose(): void {
      this.stop();
      this.audioContext.close();
    }
  }
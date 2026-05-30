import { AnecdoctOptions } from "./anecdoctOptions";
import { AnecdoctExec } from "./exec";
import { Thread } from "./thread";
import { ThreadOptions } from "./threadOptions";

/**
 * Anecdoct is the main class for interacting with the Anecdoct agent.
 *
 * Use the `startThread()` method to start a new thread or `resumeThread()` to resume a previously started thread.
 */
export class Anecdoct {
  private exec: AnecdoctExec;
  private options: AnecdoctOptions;

  constructor(options: AnecdoctOptions = {}) {
    const { anecdoctPathOverride, env, config } = options;
    this.exec = new AnecdoctExec(anecdoctPathOverride, env, config);
    this.options = options;
  }

  /**
   * Starts a new conversation with an agent.
   * @returns A new thread instance.
   */
  startThread(options: ThreadOptions = {}): Thread {
    return new Thread(this.exec, this.options, options);
  }

  /**
   * Resumes a conversation with an agent based on the thread id.
   * Threads are persisted in ~/.anecdoct/sessions.
   *
   * @param id The id of the thread to resume.
   * @returns A new thread instance.
   */
  resumeThread(id: string, options: ThreadOptions = {}): Thread {
    return new Thread(this.exec, this.options, options, id);
  }
}

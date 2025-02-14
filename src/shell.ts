import { IPCBase, CommandResult } from "./ipc";

export class Command {
    private pid = "";
    private program;
    private args?: string[];
    private ipc = new IPCBase();

    constructor(program: string, args?: string[]) {
        this.program = program;
        this.args = args;
    }

    async spawn(): Promise<CommandResult> {
        this.pid = window.crypto.randomUUID();
        return await this.ipc.invoke("spawn", {
            program: this.program,
            args: this.args,
            cancellation_token: this.pid,
        });
    }

    async kill() {
        if (this.pid) {
            return await this.ipc.invoke("kill", this.pid);
        }
    }
}

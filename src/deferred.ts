export default class Deferred<T> {
    promise: Promise<T>;
    resolve: (value: T | PromiseLike<T>) => void = () => {
        /**/
    };
    reject: (reason?: any) => void = () => {
        /**/
    };

    constructor() {
        this.promise = new Promise<T>((resolve, reject) => {
            this.reject = reject;
            this.resolve = resolve;
        });
    }
}

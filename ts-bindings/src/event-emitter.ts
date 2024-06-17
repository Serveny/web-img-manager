/** Helper Class for event emitting */
export class EventEmitter {
  listeners: Listener<Event>[] = [];

  emit(eventName: string, event: Event) {
    this.listeners
      .filter(({ name }) => name === eventName)
      .forEach(({ callback }) => callback(event), 0);
  }

  on(name: string, callback: (ev: Event) => void) {
    if (typeof callback === 'function' && typeof name === 'string') {
      this.listeners.push({ name, callback });
    }
  }

  off(eventName: string, callback: (ev: Event) => void) {
    this.listeners = this.listeners.filter(
      (listener) =>
        !(listener.name === eventName && listener.callback === callback)
    );
  }

  destroy() {
    this.listeners.length = 0;
  }
}

class Listener<Ev> {
  constructor(public name: string, public callback: (ev: Ev) => void) {}
}

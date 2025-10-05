
const blitzConsole = {
  log: function(message) {
    __blitz__.internal_print(message + "\n");
  }
}

globalThis.console = blitzConsole;

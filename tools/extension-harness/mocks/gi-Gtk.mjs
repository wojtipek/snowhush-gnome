const Gtk = {
    StringList: {new(items) { return {items: [...items], splice(pos, n, add) { this.items.splice(pos, n, ...add); }, get_string(i) { return this.items[i]; }}; }},
};
export default Gtk;

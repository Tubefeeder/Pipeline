use relm::StreamHandle;

/// A builder for the list elements of `LazyList`.
pub trait ListElementBuilder<W: relm::Widget> {
    /// Return the next batch of list elements to insert into the list.
    fn poll(&mut self) -> Vec<W::ModelParam>;

    /// Used for sending messages to the created widgets.
    fn add_stream(&mut self, _stream: StreamHandle<W::Msg>) {}

    /// Get the signal to emit to the clicked row.
    /// None if no signal should be sent.
    fn get_clicked_signal(&self) -> Option<W::Msg> {
        None
    }
}

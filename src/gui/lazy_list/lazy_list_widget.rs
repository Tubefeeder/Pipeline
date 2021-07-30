/*
 * Copyright 2021 Julian Schmidhuber <github@schmiddi.anonaddy.com>
 *
 * This file is part of Tubefeeder.
 *
 * Tubefeeder is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * Tubefeeder is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with Tubefeeder.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use super::list_element_builder::ListElementBuilder;

use gtk::prelude::*;
use gtk::{Adjustment, ListBoxRow, PositionType, SelectionMode};
use relm::{connect, Component, ContainerWidget, Relm, Update, Widget};
use relm_derive::Msg;

#[derive(Msg)]
pub enum LazyListMsg<W: relm::Widget> {
    EdgeReached(PositionType),
    SetListElementBuilder(Box<dyn ListElementBuilder<W>>),
    RowActivated(ListBoxRow),
    LoadMore,
}

pub struct LazyListModel<W: 'static + relm::Widget> {
    builder: Option<Box<dyn ListElementBuilder<W>>>,
    elements: Vec<Component<W>>,
    relm: Relm<LazyList<W>>,
}

/// A generic list widget only loading more rows when needed.
/// These rows must all have the same relm widget type, the generic type W of this widget.
/// The rows are created using `ListElementBuilder.poll`.
/// You can set the used builder by emitting the signal `LazyListMsg::SetListElementBuilder`.
/// You can also set a signal to be sent to a clicked row with `ListElementBuilder.get_clicked_signal`.
pub struct LazyList<W: 'static + relm::Widget> {
    model: LazyListModel<W>,
    window: gtk::ScrolledWindow,
    list: gtk::ListBox,
}

impl<W: 'static + relm::Widget> Update for LazyList<W> {
    type Model = LazyListModel<W>;
    type ModelParam = ();
    type Msg = LazyListMsg<W>;

    fn model(relm: &Relm<Self>, _: ()) -> LazyListModel<W> {
        LazyListModel::<W> {
            builder: None,
            elements: Vec::new(),
            relm: relm.clone(),
        }
    }

    fn update(&mut self, event: LazyListMsg<W>) {
        match event {
            LazyListMsg::EdgeReached(edge) => {
                if edge == PositionType::Bottom {
                    self.model.relm.stream().emit(LazyListMsg::LoadMore);
                }
            }
            LazyListMsg::SetListElementBuilder(builder) => {
                self.model.builder = Some(builder);
                self.model.elements = vec![];

                let list_clone = self.list.clone();
                self.list.forall(|w| list_clone.remove(w));

                self.model.relm.stream().emit(LazyListMsg::LoadMore);
            }
            LazyListMsg::RowActivated(row) => {
                let index = self
                    .list
                    .get_children()
                    .iter()
                    .position(|x| x.clone() == row)
                    .unwrap();

                let clicked = &self.model.elements[index];

                if self.model.builder.is_some() {
                    if let Some(msg) = self.model.builder.as_ref().unwrap().get_clicked_signal() {
                        clicked.emit(msg);
                    }
                }
            }
            LazyListMsg::LoadMore => {
                if self.model.builder.is_some() {
                    let builder = self.model.builder.as_mut().unwrap();
                    let new_entries: Vec<W::ModelParam> = builder.poll();
                    for entry in new_entries {
                        let widget = self.list.add_widget::<W>(entry);

                        let stream = widget.stream();
                        builder.add_stream(stream);

                        self.model.elements.push(widget);
                    }
                }
            }
        }
    }
}

impl<W: 'static + relm::Widget> Widget for LazyList<W> {
    type Root = gtk::ScrolledWindow;

    fn root(&self) -> Self::Root {
        self.window.clone()
    }

    fn view(relm: &Relm<Self>, model: Self::Model) -> Self {
        let window = gtk::ScrolledWindow::new::<Adjustment, Adjustment>(None, None);

        window.set_hexpand(true);
        window.set_vexpand(true);

        connect!(
            relm,
            window,
            connect_edge_reached(_, edge),
            LazyListMsg::EdgeReached(edge)
        );

        let viewport = gtk::Viewport::new::<Adjustment, Adjustment>(None, None);
        let list = gtk::ListBox::new();
        list.set_selection_mode(SelectionMode::None);

        connect!(
            relm,
            list,
            connect_row_activated(_, row),
            LazyListMsg::RowActivated(row.clone())
        );

        viewport.add(&list);
        window.add(&viewport);

        window.show_all();

        LazyList::<W> {
            model,
            window,
            list,
        }
    }
}

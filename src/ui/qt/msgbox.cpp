#include "msgbox.hpp"

static std::unique_ptr<QMessageBox> new_message_box_impl(QWidget *parent) {
    auto box = std::make_unique<QMessageBox>(parent);
    box->setWindowModality(Qt::WindowModal);
    return box;
}

std::unique_ptr<QMessageBox> new_message_box() {
    return new_message_box_impl(nullptr);
}

std::unique_ptr<QMessageBox> new_message_box(QWidget &parent) {
    return new_message_box_impl(&parent);
}

void message_box_connect_finished(QMessageBox &b,
                                  callback_fn_t<void(int)> callback,
                                  std::uint8_t const *data) {
    QObject::connect(&b, &QMessageBox::finished,
                     [callback, data](int result) { callback(data, result); });
}

void message_box_set_texts(QMessageBox &b, rust::Str title, rust::Str msg,
                           rust::Str instr) {
    b.setWindowTitle(QString::fromUtf8(title.data(), title.size()));
    if (instr.empty()) {
        b.setText(QString::fromUtf8(msg.data(), msg.size()));
    } else {
        b.setText(QString::fromUtf8(instr.data(), instr.size()));
        b.setInformativeText(QString::fromUtf8(msg.data(), msg.size()));
    }
}

QPushButton *message_box_add_button(QMessageBox &b, rust::Str text) {
    return b.addButton(QString::fromUtf8(text.data(), text.size()),
                       QMessageBox::AcceptRole);
}

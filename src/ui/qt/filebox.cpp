#include "filebox.hpp"

static std::unique_ptr<QFileDialog> new_file_dialog_impl(QWidget *parent) {
    auto box = std::make_unique<QFileDialog>(parent);
    box->setWindowModality(Qt::WindowModal);
    return box;
}

std::unique_ptr<QFileDialog> new_file_dialog() {
    return new_file_dialog_impl(nullptr);
}

std::unique_ptr<QFileDialog> new_file_dialog(QWidget &parent) {
    return new_file_dialog_impl(&parent);
}

void file_dialog_connect_finished(QFileDialog &b,
                                  callback_fn_t<void(int)> callback,
                                  std::uint8_t const *data) {
    QObject::connect(&b, &QFileDialog::finished,
                     [callback, data](int result) { callback(data, result); });
}

void file_dialog_set_texts(QFileDialog &b, rust::Str title, rust::Str filename,
                           rust::Str filter) {
    b.setWindowTitle(QString::fromUtf8(title.data(), title.size()));
    if (!filename.empty()) {
        b.selectFile(QString::fromUtf8(filename.data(), filename.size()));
    }
    if (!filter.empty()) {
        b.setNameFilter(QString::fromUtf8(filter.data(), filter.size()));
    }
}

rust::Vec<rust::String> file_dialog_files(QFileDialog const &b) {
    rust::Vec<rust::String> results{};
    for (QString &f : b.selectedFiles()) {
        results.push_back(
            rust::String{(const char16_t *)f.utf16(), (std::size_t)f.size()});
    }
    return results;
}

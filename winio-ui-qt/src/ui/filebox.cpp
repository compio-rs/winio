#include "filebox.hpp"

std::unique_ptr<QFileDialog> new_file_dialog(QWidget *parent) {
    auto box = std::make_unique<QFileDialog>(parent);
    box->setWindowModality(Qt::WindowModal);
    return box;
}

void file_dialog_connect_finished(QFileDialog &b,
                                  callback_fn_t<void(int)> callback,
                                  std::uint8_t const *data) {
    QObject::connect(&b, &QFileDialog::finished,
                     [callback, data](int result) { callback(data, result); });
}

rust::Vec<rust::String> file_dialog_files(QFileDialog const &b) {
    rust::Vec<rust::String> results{};
    for (QString &f : b.selectedFiles()) {
        results.push_back(
            rust::String{(const char16_t *)f.utf16(), (std::size_t)f.size()});
    }
    return results;
}

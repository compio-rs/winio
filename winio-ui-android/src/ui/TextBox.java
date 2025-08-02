package rs.compio.winio;

import android.text.InputType;

public class TextBox extends Edit {
    TextBox(Window parent) {
        super(parent);
        setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_MULTI_LINE);
    }
}
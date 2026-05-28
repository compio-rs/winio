package rs.compio.winio;

import android.text.TextPaint;
import android.view.View;
import java.lang.Runnable;

public class ClickableSpan extends android.text.style.ClickableSpan {
    private final Runnable onClick;

    public ClickableSpan() {
        this.onClick = null;
    }

    public void setOnClick(Runnable onClick) {
        this.onClick = onClick;
    }

    @Override
    public void onClick(View widget) {
        if (onClick != null) {
            onClick.run();
        }
    }
}

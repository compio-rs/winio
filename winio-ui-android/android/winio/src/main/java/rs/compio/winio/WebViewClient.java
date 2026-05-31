package rs.compio.winio;

import android.graphics.Bitmap;
import android.webkit.WebView;

public class WebViewClient extends android.webkit.WebViewClient {
    private Runnable on_page_started;
    private Runnable on_page_finished;

    public void setOnPageStarted(Runnable r) {
        this.on_page_started = r;
    }

    public void setOnPageFinished(Runnable r) {
        this.on_page_finished = r;
    }

    @Override
    public void onPageStarted(WebView view, String url, Bitmap favicon) {
        super.onPageStarted(view, url, favicon);
        if (on_page_started != null) {
            on_page_started.run();
        }
    }

    @Override
    public void onPageFinished(WebView view, String url) {
        super.onPageFinished(view, url);
        if (on_page_finished != null) {
            on_page_finished.run();
        }
    }
}

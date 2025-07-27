package rs.compio.winio;

import android.graphics.Paint;
import android.graphics.Path;
import android.graphics.RectF;
import android.graphics.Bitmap;
import android.graphics.BitmapFactory;
import android.graphics.LinearGradient;
import android.graphics.RadialGradient;
import android.graphics.Shader;
import android.graphics.Color;
import android.graphics.Matrix;
import android.graphics.Typeface;
import android.view.View;
import android.widget.FrameLayout;
import java.io.ByteArrayOutputStream;
import java.io.ByteArrayInputStream;

/**
 * Canvas widget for drawing operations
 */
public class Canvas extends View {
    private Widget widget;
    private Bitmap canvasBitmap;

    /**
     * Creates a new Canvas
     * @param parent The parent window
     */
    public Canvas(Window parent) {
        super(parent.getContext());
        parent.addView(this);
        this.widget = new Widget(this);
    }

    @Override
    protected void onDraw(android.graphics.Canvas canvas) {
        super.onDraw(canvas);
        if (canvasBitmap != null)
            canvas.drawBitmap(canvasBitmap, 0, 0, null);
    }

    /**
     * Sets the visibility of the canvas
     * @param visible True to make visible, false to hide
     */
    public void setVisible(boolean visible) {
        this.widget.setVisible(visible);
    }

    /**
     * Checks if the canvas is visible
     * @return True if visible, false otherwise
     */
    public boolean isVisible() {
        return this.widget.isVisible();
    }

    /**
     * Sets the size of the canvas
     * @param width Width in pixels
     * @param height Height in pixels
     */
    public void setSize(double width, double height) {
        this.widget.setSize(width, height);
    }

    /**
     * Gets the size of the canvas
     * @return Array with [width, height]
     */
    public double[] getSize() {
        return this.widget.getSize();
    }

    /**
     * Sets the location of the canvas
     * @param x X coordinate
     * @param y Y coordinate
     */
    public void setLoc(double x, double y) {
        this.widget.setLoc(x, y);
    }

    /**
     * Gets the location of the canvas
     * @return Array with [x, y]
     */
    public double[] getLoc() {
        return this.widget.getLoc();
    }

    /**
     * Creates a drawing context for this canvas
     * @return A new DrawingContext
     */
    public DrawingContext context() {
        double[] size = widget.getSize();
        canvasBitmap = Bitmap.createBitmap((int)size[0], (int)size[1], Bitmap.Config.ARGB_8888);
        return new DrawingContext(this, canvasBitmap);
    }

    /**
     * Interface for drawing brushes
     */
    public interface Brush {
        Paint getPaint();
    }

    /**
     * Interface for drawing pens
     */
    public interface Pen {
        Paint getPaint();
    }

    /**
     * Solid color brush implementation
     */
    public static class SolidColorBrush implements Brush {
        private Paint paint;

        public SolidColorBrush(int color) {
            paint = new Paint();
            paint.setColor(color);
            paint.setStyle(Paint.Style.FILL);
        }

        @Override
        public Paint getPaint() {
            return paint;
        }
    }

    /**
     * Linear gradient brush implementation
     */
    public static class LinearGradientBrush implements Brush {
        private Paint paint;

        public LinearGradientBrush(float x1, float y1, float x2, float y2, int[] colors, float[] positions) {
            paint = new Paint();
            paint.setShader(new LinearGradient(x1, y1, x2, y2, colors, positions, Shader.TileMode.CLAMP));
            paint.setStyle(Paint.Style.FILL);
        }

        @Override
        public Paint getPaint() {
            return paint;
        }
    }

    /**
     * Radial gradient brush implementation
     */
    public static class RadialGradientBrush implements Brush {
        private Paint paint;

        public RadialGradientBrush(float centerX, float centerY, float radius, int[] colors, float[] positions) {
            paint = new Paint();
            paint.setShader(new RadialGradient(centerX, centerY, radius, colors, positions, Shader.TileMode.CLAMP));
            paint.setStyle(Paint.Style.FILL);
        }

        @Override
        public Paint getPaint() {
            return paint;
        }
    }

    /**
     * Brush pen implementation
     */
    public static class BrushPen implements Pen {
        private Paint paint;

        public BrushPen(Brush brush, float strokeWidth) {
            paint = new Paint(brush.getPaint());
            paint.setStyle(Paint.Style.STROKE);
            paint.setStrokeWidth(strokeWidth);
        }

        @Override
        public Paint getPaint() {
            return paint;
        }
    }

    /**
     * Drawing path for complex shapes
     */
    public static class DrawingPath {
        private Path path;

        public DrawingPath(Path path) {
            this.path = path;
        }

        public Path getPath() {
            return path;
        }
    }

    /**
     * Builder for creating drawing paths
     */
    public static class DrawingPathBuilder {
        private Path path;

        public DrawingPathBuilder(float startX, float startY) {
            path = new Path();
            path.moveTo(startX, startY);
        }

        /**
         * Adds a line to the path
         * @param x X coordinate
         * @param y Y coordinate
         */
        public void addLine(float x, float y) {
            path.lineTo(x, y);
        }

        /**
         * Adds an arc to the path
         * @param centerX Center X coordinate
         * @param centerY Center Y coordinate
         * @param radiusX X radius
         * @param radiusY Y radius
         * @param startAngle Start angle in degrees
         * @param endAngle End angle in degrees
         * @param clockwise True for clockwise, false for counter-clockwise
         */
        public void addArc(float centerX, float centerY, float radiusX, float radiusY, 
                          float startAngle, float endAngle, boolean clockwise) {
            RectF oval = new RectF(centerX - radiusX, centerY - radiusY, 
                                  centerX + radiusX, centerY + radiusY);
            float sweepAngle = clockwise ? endAngle - startAngle : startAngle - endAngle;
            path.arcTo(oval, startAngle, sweepAngle);
        }

        /**
         * Adds a bezier curve to the path
         * @param x1 First control point X
         * @param y1 First control point Y
         * @param x2 Second control point X
         * @param y2 Second control point Y
         * @param x3 End point X
         * @param y3 End point Y
         */
        public void addBezier(float x1, float y1, float x2, float y2, float x3, float y3) {
            path.cubicTo(x1, y1, x2, y2, x3, y3);
        }

        /**
         * Builds the path
         * @param close True to close the path
         * @return The completed DrawingPath
         */
        public DrawingPath build(boolean close) {
            if (close) {
                path.close();
            }
            return new DrawingPath(path);
        }
    }

    /**
     * Font for drawing text
     */
    public static class DrawingFont {
        private Typeface typeface;
        private float size;

        public DrawingFont(Typeface typeface, float size) {
            this.typeface = typeface;
            this.size = size;
        }

        public Typeface getTypeface() {
            return typeface;
        }

        public float getSize() {
            return size;
        }
    }

    /**
     * Image for drawing on canvas
     */
    public static class DrawingImage {
        private Bitmap bitmap;

        public DrawingImage(Bitmap bitmap) {
            this.bitmap = bitmap;
        }

        public Bitmap getBitmap() {
            return bitmap;
        }

        /**
         * Gets the size of the image
         * @return Array with [width, height]
         */
        public double[] getSize() {
            return new double[] { bitmap.getWidth(), bitmap.getHeight() };
        }
    }

    /**
     * Point representation
     */
    public static class Point {
        public float x;
        public float y;

        public Point(float x, float y) {
            this.x = x;
            this.y = y;
        }
    }

    /**
     * Rectangle representation
     */
    public static class Rect {
        public float x;
        public float y;
        public float width;
        public float height;

        public Rect(float x, float y, float width, float height) {
            this.x = x;
            this.y = y;
            this.width = width;
            this.height = height;
        }

        public RectF toRectF() {
            return new RectF(x, y, x + width, y + height);
        }
    }

    /**
     * Size representation
     */
    public static class Size {
        public float width;
        public float height;

        public Size(float width, float height) {
            this.width = width;
            this.height = height;
        }
    }

    /**
     * Drawing context for performing drawing operations
     */
    public class DrawingContext {
        private View view;
        private android.graphics.Canvas canvas;

        /**
         * Creates a new drawing context
         * @param view The view to draw on
         */
        public DrawingContext(View view, Bitmap canvasBitmap) {
            this.view = view;
            this.canvas = new android.graphics.Canvas(canvasBitmap);
        }

        /**
         * Draws a path with the specified pen
         * @param pen The pen to use
         * @param path The path to draw
         */
        public void drawPath(Pen pen, DrawingPath path) {
            canvas.drawPath(path.getPath(), pen.getPaint());
            view.invalidate();
        }

        /**
         * Fills a path with the specified brush
         * @param brush The brush to use
         * @param path The path to fill
         */
        public void fillPath(Brush brush, DrawingPath path) {
            canvas.drawPath(path.getPath(), brush.getPaint());
            view.invalidate();
        }

        /**
         * Draws an arc with the specified pen
         * @param pen The pen to use
         * @param rect The bounding rectangle
         * @param start The start angle in degrees
         * @param end The end angle in degrees
         */
        public void drawArc(Pen pen, Rect rect, float start, float end) {
            canvas.drawArc(rect.toRectF(), start, end - start, false, pen.getPaint());
            view.invalidate();
        }

        /**
         * Draws a pie slice with the specified pen
         * @param pen The pen to use
         * @param rect The bounding rectangle
         * @param start The start angle in degrees
         * @param end The end angle in degrees
         */
        public void drawPie(Pen pen, Rect rect, float start, float end) {
            canvas.drawArc(rect.toRectF(), start, end - start, true, pen.getPaint());
            view.invalidate();
        }

        /**
         * Fills a pie slice with the specified brush
         * @param brush The brush to use
         * @param rect The bounding rectangle
         * @param start The start angle in degrees
         * @param end The end angle in degrees
         */
        public void fillPie(Brush brush, Rect rect, float start, float end) {
            Paint paint = brush.getPaint();
            paint.setStyle(Paint.Style.FILL);
            canvas.drawArc(rect.toRectF(), start, end - start, true, paint);
            view.invalidate();
        }

        /**
         * Draws an ellipse with the specified pen
         * @param pen The pen to use
         * @param rect The bounding rectangle
         */
        public void drawEllipse(Pen pen, Rect rect) {
            canvas.drawOval(rect.toRectF(), pen.getPaint());
            view.invalidate();
        }

        /**
         * Fills an ellipse with the specified brush
         * @param brush The brush to use
         * @param rect The bounding rectangle
         */
        public void fillEllipse(Brush brush, Rect rect) {
            canvas.drawOval(rect.toRectF(), brush.getPaint());
            view.invalidate();
        }

        /**
         * Draws a line with the specified pen
         * @param pen The pen to use
         * @param start The start point
         * @param end The end point
         */
        public void drawLine(Pen pen, Point start, Point end) {
            canvas.drawLine(start.x, start.y, end.x, end.y, pen.getPaint());
            view.invalidate();
        }

        /**
         * Draws a rectangle with the specified pen
         * @param pen The pen to use
         * @param rect The rectangle to draw
         */
        public void drawRect(Pen pen, Rect rect) {
            canvas.drawRect(rect.toRectF(), pen.getPaint());
            view.invalidate();
        }

        /**
         * Fills a rectangle with the specified brush
         * @param brush The brush to use
         * @param rect The rectangle to fill
         */
        public void fillRect(Brush brush, Rect rect) {
            canvas.drawRect(rect.toRectF(), brush.getPaint());
            view.invalidate();
        }

        /**
         * Draws a rounded rectangle with the specified pen
         * @param pen The pen to use
         * @param rect The rectangle to draw
         * @param round The corner radius
         */
        public void drawRoundRect(Pen pen, Rect rect, Size round) {
            canvas.drawRoundRect(rect.toRectF(), round.width, round.height, pen.getPaint());
            view.invalidate();
        }

        /**
         * Fills a rounded rectangle with the specified brush
         * @param brush The brush to use
         * @param rect The rectangle to fill
         * @param round The corner radius
         */
        public void fillRoundRect(Brush brush, Rect rect, Size round) {
            canvas.drawRoundRect(rect.toRectF(), round.width, round.height, brush.getPaint());
            view.invalidate();
        }

        /**
         * Draws a string with the specified brush and font
         * @param brush The brush to use
         * @param text The text to draw
         * @param x X coordinate
         * @param y Y coordinate
         * @param font The font to use
         */
        public void drawStr(Brush brush, String text, float x, float y, DrawingFont font) {
            Paint paint = new Paint(brush.getPaint());
            paint.setTypeface(font.getTypeface());
            paint.setTextSize(font.getSize());
            canvas.drawText(text, x, y, paint);
            view.invalidate();
        }

        /**
         * Creates an image from raw bytes
         * @param bytes The image bytes
         * @return A new DrawingImage
         */
        public DrawingImage createImage(byte[] bytes) {
            Bitmap bitmap = BitmapFactory.decodeByteArray(bytes, 0, bytes.length);
            return new DrawingImage(bitmap);
        }

        /**
         * Draws an image at the specified location
         * @param image The image to draw
         * @param x X coordinate
         * @param y Y coordinate
         */
        public void drawImage(DrawingImage image, float x, float y) {
            canvas.drawBitmap(image.getBitmap(), x, y, null);
            view.invalidate();
        }

        /**
         * Creates a path builder
         * @param startX Starting X coordinate
         * @param startY Starting Y coordinate
         * @return A new DrawingPathBuilder
         */
        public DrawingPathBuilder createPathBuilder(float startX, float startY) {
            return new DrawingPathBuilder(startX, startY);
        }
    }
}
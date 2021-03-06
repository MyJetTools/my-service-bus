
var gulp = require('gulp');
var minifyjs = require('gulp-js-minify');
var concat = require('gulp-concat');

gulp.task('default', function () {
    return gulp
        .src([
            './JavaScript/SubpagesWidget.js',
            './JavaScript/PlugIcon.js',
            './JavaScript/Utils.js',
            './JavaScript/Iterators.js',
            './JavaScript/ServiceLocator.js',
            './JavaScript/HtmlGraph.js',
            './JavaScript/HtmlSessions.js',
            './JavaScript/HtmlStatusBar.js',
            './JavaScript/HtmlMain.js',
            './JavaScript/HtmlTopics.js',
            './JavaScript/HtmlQueue.js',

            './JavaScript/main.js'
        ])
        .pipe(minifyjs())
        .pipe(concat('app.js'))
        .pipe(gulp.dest('./wwwroot/js/'))
});
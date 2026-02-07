// 页面滚动视图
// 支持多页面水平或垂直滚动

import 'package:flutter/material.dart';

import '../view_models/layout_view_model.dart';

/// 页面滚动方向
enum PageScrollDirection {
  vertical, // 垂直滚动（类似 Word）
  horizontal, // 水平滚动
  continuous, // 连续滚动（类似代码编辑器）
}

/// 页面视图配置
class PageViewConfig {
  final PageScrollDirection direction;
  final bool enablePageJump;
  final bool showPageIndicator;
  final double pageSpacing;
  final double minZoom;
  final double maxZoom;
  final double defaultZoom;

  const PageViewConfig({
    this.direction = PageScrollDirection.vertical,
    this.enablePageJump = true,
    this.showPageIndicator = false,
    this.pageSpacing = 20.0,
    this.minZoom = 0.5,
    this.maxZoom = 2.0,
    this.defaultZoom = 1.0,
  });
}

/// 页面滚动视图
class VelumPageView extends StatefulWidget {
  final LayoutViewModel viewModel;
  final PageViewConfig config;
  final Widget Function(int pageNumber, RenderedPage page)? pageBuilder;
  final Widget? header;
  final Widget? footer;
  final VoidCallback? onPageChanged;

  const VelumPageView({
    super.key,
    required this.viewModel,
    this.config = const PageViewConfig(),
    this.pageBuilder,
    this.header,
    this.footer,
    this.onPageChanged,
  });

  @override
  State<VelumPageView> createState() => _VelumPageViewState();
}

class _VelumPageViewState extends State<VelumPageView> {
  // 页面控制器
  late PageController _pageController;

  // 当前页面
  int _currentPage = 0;

  // 缩放比例
  double _zoom = 1.0;

  // 滚动位置
  final ScrollController _scrollController = ScrollController();

  @override
  void initState() {
    super.initState();

    _pageController = PageController(
      initialPage: 0,
      keepPage: true,
    );

    _zoom = widget.config.defaultZoom;
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _scrollController.addListener(_onScroll);
  }

  @override
  void dispose() {
    _scrollController.removeListener(_onScroll);
    _pageController.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _onScroll() {
    // 处理滚动事件
    widget.viewModel.updateVisibleRange(
      VisibleRange(
        start: _scrollController.offset,
        end: _scrollController.offset + _scrollController.position.viewportDimension,
      ),
    );
  }

  // ==================== 构建 ====================

  @override
  Widget build(BuildContext context) {
    switch (widget.config.direction) {
      case PageScrollDirection.vertical:
        return _buildVerticalPageView();
      case PageScrollDirection.horizontal:
        return _buildHorizontalPageView();
      case PageScrollDirection.continuous:
        return _buildContinuousView();
    }
  }

  /// 垂直页面滚动视图（类似 Word）
  Widget _buildVerticalPageView() {
    return Stack(
      children: [
        // 主内容
        NotificationListener<ScrollNotification>(
          onNotification: _handleScrollNotification,
          child: SingleChildScrollView(
            controller: _scrollController,
            child: Center(
              child: Column(
                children: [
                  // 页眉
                  if (widget.header != null) widget.header!,

                  // 页面列表
                  ..._buildPageList(),

                  // 页脚
                  if (widget.footer != null) widget.footer!,
                ],
              ),
            ),
          ),
        ),

        // 页面指示器
        if (widget.config.showPageIndicator) _buildPageIndicator(),
      ],
    );
  }

  /// 水平页面滚动视图
  Widget _buildHorizontalPageView() {
    return Stack(
      children: [
        // 主内容
        PageView.builder(
          controller: _pageController,
          itemCount: widget.viewModel.pageCount,
          onPageChanged: _onPageChanged,
          itemBuilder: (context, index) {
            final page = widget.viewModel.pages[index];
            return _buildPage(page, index);
          },
        ),

        // 页面指示器
        if (widget.config.showPageIndicator) _buildPageIndicator(),
      ],
    );
  }

  /// 连续滚动视图（类似代码编辑器）
  Widget _buildContinuousView() {
    return SingleChildScrollView(
      controller: _scrollController,
      child: Center(
        child: Column(
          children: [
            // 页眉
            if (widget.header != null) widget.header!,

            // 单个连续页面
            _buildContinuousDocument(),

            // 页脚
            if (widget.footer != null) widget.footer!,
          ],
        ),
      ),
    );
  }

  /// 构建页面列表
  List<Widget> _buildPageList() {
    final pages = widget.viewModel.pages;
    final pageWidth = widget.viewModel.config.width * _zoom;
    final pageHeight = widget.viewModel.config.height * _zoom;

    return pages.asMap().entries.map((entry) {
      final index = entry.key;
      final page = entry.value;

      return Padding(
        padding: EdgeInsets.symmetric(vertical: widget.config.pageSpacing),
        child: _buildPage(page, index, width: pageWidth, height: pageHeight),
      );
    }).toList();
  }

  /// 构建单个页面
  Widget _buildPage(RenderedPage page, int pageNumber,
      {double? width, double? height}) {
    final pageWidth = width ?? widget.viewModel.config.width;
    final pageHeight = height ?? widget.viewModel.config.height;

    return SizedBox(
      width: pageWidth,
      height: pageHeight,
      child: Container(
        decoration: BoxDecoration(
          color: Colors.white,
          boxShadow: [
            BoxShadow(
              color: Colors.black.withOpacity(0.1),
              blurRadius: 4,
              offset: const Offset(0, 2),
            ),
          ],
        ),
        child: CustomPaint(
          size: Size(pageWidth, pageHeight),
          painter: PagePainter(
            page: page,
            zoom: _zoom,
          ),
        ),
      ),
    );
  }

  /// 构建连续文档视图
  Widget _buildContinuousDocument() {
    final documentWidth = widget.viewModel.documentWidth * _zoom;
    final documentHeight = widget.viewModel.documentHeight * _zoom;

    return SizedBox(
      width: documentWidth,
      height: documentHeight,
      child: CustomPaint(
        size: Size(documentWidth, documentHeight),
        painter: ContinuousDocumentPainter(
          viewModel: widget.viewModel,
          zoom: _zoom,
        ),
      ),
    );
  }

  /// 构建页面指示器
  Widget _buildPageIndicator() {
    return Positioned(
      bottom: 20,
      left: 0,
      right: 0,
      child: Center(
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          decoration: BoxDecoration(
            color: Colors.black54,
            borderRadius: BorderRadius.circular(16),
          ),
          child: Text(
            '${_currentPage + 1} / ${widget.viewModel.pageCount}',
            style: const TextStyle(
              color: Colors.white,
              fontSize: 14,
            ),
          ),
        ),
      ),
    );
  }

  /// 处理滚动通知
  bool _handleScrollNotification(ScrollNotification notification) {
    if (notification is ScrollEndNotification) {
      // 计算当前页面
      final pageHeight = widget.viewModel.config.height +
          widget.config.pageSpacing * 2;
      final newPage = (notification.metrics.pixels / pageHeight).round();

      if (newPage != _currentPage) {
        setState(() {
          _currentPage = newPage;
        });
        widget.onPageChanged?.call();
      }
    }
    return false;
  }

  /// 页面改变回调
  void _onPageChanged(int page) {
    setState(() {
      _currentPage = page;
    });
    widget.onPageChanged?.call();
  }

  // ==================== 导航操作 ====================

  /// 跳转到指定页面
  void jumpToPage(int page) {
    if (widget.config.direction == PageScrollDirection.horizontal) {
      _pageController.jumpToPage(page);
    } else {
      final pageHeight = widget.viewModel.config.height +
          widget.config.pageSpacing * 2;
      _scrollController.jumpTo(page * pageHeight);
    }
  }

  /// 动画跳转到页面
  Future<void> animateToPage(int page, {Duration duration = const Duration(milliseconds: 300)}) async {
    if (widget.config.direction == PageScrollDirection.horizontal) {
      await _pageController.animateToPage(
        page,
        duration: duration,
        curve: Curves.easeInOut,
      );
    } else {
      final pageHeight = widget.viewModel.config.height +
          widget.config.pageSpacing * 2;
      await _scrollController.animateTo(
        page * pageHeight,
        duration: duration,
        curve: Curves.easeInOut,
      );
    }
  }

  /// 跳转到首页
  void jumpToFirstPage() {
    jumpToPage(0);
  }

  /// 跳转到末页
  void jumpToLastPage() {
    jumpToPage(widget.viewModel.pageCount - 1);
  }

  /// 上一页
  void previousPage() {
    if (_currentPage > 0) {
      jumpToPage(_currentPage - 1);
    }
  }

  /// 下一页
  void nextPage() {
    if (_currentPage < widget.viewModel.pageCount - 1) {
      jumpToPage(_currentPage + 1);
    }
  }

  // ==================== 缩放操作 ====================

  /// 设置缩放比例
  void setZoom(double zoom) {
    final clampedZoom = zoom.clamp(widget.config.minZoom, widget.config.maxZoom);
    setState(() {
      _zoom = clampedZoom;
    });
  }

  /// 放大
  void zoomIn() {
    setZoom(_zoom + 0.1);
  }

  /// 缩小
  void zoomOut() {
    setZoom(_zoom - 0.1);
  }

  /// 重置缩放
  void resetZoom() {
    setZoom(widget.config.defaultZoom);
  }

  /// 适应页面
  void fitToPage() {
    // TODO: 计算合适的缩放比例
  }

  /// 适应宽度
  void fitToWidth() {
    // TODO: 计算适应宽度的缩放比例
  }
}

/// 页面绘制器
class PagePainter extends CustomPainter {
  final RenderedPage page;
  final double zoom;

  PagePainter({required this.page, this.zoom = 1.0});

  @override
  void paint(Canvas canvas, Size size) {
    canvas.save();
    canvas.scale(zoom, zoom);

    // 绘制页面内容
    for (final line in page.lines) {
      _drawLine(canvas, line);
    }

    canvas.restore();
  }

  void _drawLine(Canvas canvas, RenderedLine line) {
    for (final span in line.spans) {
      _drawSpan(canvas, span, line.y);
    }
  }

  void _drawSpan(Canvas canvas, RenderedSpan span, double baseline) {
    final textPainter = TextPainter(
      text: TextSpan(
        text: span.text,
        style: TextStyle(
          fontSize: span.style.fontSize ?? 12,
          fontFamily: span.style.fontFamily,
          fontWeight: span.style.bold ? FontWeight.bold : FontWeight.normal,
          fontStyle: span.style.italic ? FontStyle.italic : FontStyle.normal,
          decoration: span.style.underline ? TextDecoration.underline : null,
        ),
      ),
      textDirection: TextDirection.ltr,
    );

    textPainter.layout();

    canvas.save();
    canvas.translate(span.x, baseline);
    textPainter.paint(canvas, Offset.zero);
    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant PagePainter oldDelegate) {
    return oldDelegate.page != page || oldDelegate.zoom != zoom;
  }
}

/// 连续文档绘制器
class ContinuousDocumentPainter extends CustomPainter {
  final LayoutViewModel viewModel;
  final double zoom;

  ContinuousDocumentPainter({
    required this.viewModel,
    this.zoom = 1.0,
  });

  @override
  void paint(Canvas canvas, Size size) {
    canvas.save();
    canvas.scale(zoom, zoom);

    // 绘制所有行
    for (final line in viewModel.lines) {
      for (final span in line.spans) {
        _drawSpan(canvas, span, line.y);
      }
    }

    canvas.restore();
  }

  void _drawSpan(Canvas canvas, RenderedSpan span, double baseline) {
    final textPainter = TextPainter(
      text: TextSpan(
        text: span.text,
        style: TextStyle(
          fontSize: span.style.fontSize ?? 12,
          fontFamily: span.style.fontFamily,
          fontWeight: span.style.bold ? FontWeight.bold : FontWeight.normal,
          fontStyle: span.style.italic ? FontStyle.italic : FontStyle.normal,
          decoration: span.style.underline ? TextDecoration.underline : null,
        ),
      ),
      textDirection: TextDirection.ltr,
    );

    textPainter.layout();

    canvas.save();
    canvas.translate(span.x, baseline);
    textPainter.paint(canvas, Offset.zero);
    canvas.restore();
  }

  @override
  bool shouldRepaint(covariant ContinuousDocumentPainter oldDelegate) {
    return oldDelegate.viewModel != viewModel || oldDelegate.zoom != zoom;
  }
}

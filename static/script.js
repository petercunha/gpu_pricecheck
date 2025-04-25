$(document).ready(function () {
	// Initialize DataTable
	var table = $('#gpuTable').DataTable({
		pageLength: 25, // Default number of rows
		order: [[2, 'asc']], // Default sort by price ascending
		columnDefs: [
			{ targets: [4], orderable: false }, // Disable sorting for Link column
		],
	})

	// Custom filtering function
	$.fn.dataTable.ext.search.push(function (settings, data, dataIndex) {
		var row = table.row(dataIndex).node()
		var status = $(row).data('status') // Get status from data-status attribute
		var showRow = false

		$('.status-filter:checked').each(function () {
			if ($(this).val() === status) {
				showRow = true
				return false // Break loop
			}
		})

		return showRow
	})

	// Re-draw the table when a filter checkbox changes
	$('.status-filter').on('change', function () {
		table.draw()
	})

	// Initial draw to apply default filters
	table.draw()
})
